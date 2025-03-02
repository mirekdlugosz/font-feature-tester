import argparse
import os
import subprocess
import tempfile
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent


def default_output_dir(dry_run: bool):
    if dry_run:
        return SCRIPT_DIR / "font-images"
    tmpdir = tempfile.mkdtemp(prefix="fonts-")
    return Path(tmpdir)


def find_rust_binary():
    for candidate in ("release", "debug"):
        candidate_path = SCRIPT_DIR / "target" / candidate / "font-feature-tester"
        if candidate_path.exists():
            return candidate_path


def parse_args():
    parser = argparse.ArgumentParser(
        description="Execute rust program binary for all configuration files in a directory"
    )
    parser.add_argument(
        "-i",
        "--input-path",
        type=Path,
        help=(
            "Path to file whose content will be written on image. "
            "Passed verbatim to rust --input-path"
        ),
    )
    parser.add_argument(
        "-o",
        "--output-dir",
        type=Path,
        help=(
            "Path to directory where images will be written. "
            "If not provided, new directory will be created in temporary dir"
        ),
    )
    parser.add_argument(
        "--bg-color",
        help="Image background color. Passed verbatim to rust --bg-color",
    )
    parser.add_argument(
        "--fg-color",
        help="Image text color. Passed verbatim to rust --fg-color",
    )
    parser.add_argument(
        "--image-width",
        help="Image width. Passed verbatim to rust --image-width",
    )
    parser.add_argument(
        "--image-height",
        help="Image height. Passed verbatim to rust --image-height",
    )
    parser.add_argument(
        "--configuration-dir",
        type=Path,
        help=(
            "Path to directory where configuration files in TOML format reside. "
            "Uses resources/configs/ by default"
        ),
    )
    parser.add_argument(
        "--rust-binary",
        type=Path,
        help="Path to rust program executable. Searches inside target/ by default.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Dry run mode - don't actually do anything, print commands that would be executed",
    )
    args = parser.parse_args()

    if args.input_path and not args.input_path.is_file():
        raise ValueError("--input-path must be a path to existing file")

    if not args.output_dir:
        args.output_dir = default_output_dir(args.dry_run)
    if not args.output_dir.exists() and not args.dry_run:
        args.output_dir.mkdir(parents=True)

    if not args.configuration_dir:
        args.configuration_dir = SCRIPT_DIR / "resources/configs"
    if not args.configuration_dir.is_dir():
        raise ValueError(f"Configuration dir {args.configuration_dir} does not exist")

    if not args.rust_binary:
        if not (found_binary := find_rust_binary()):
            raise ValueError("Could not find rust program binary. Did you run `cargo build`?")
        args.rust_binary = found_binary
    if not (args.rust_binary.is_file() and os.access(args.rust_binary, os.X_OK)):
        raise ValueError(f"Rust binary {args.rust_binary} is not executable file")

    return args


def binary_common_params(args):
    params = []
    if input_path := args.input_path:
        params.extend(["--input-path", input_path.resolve().as_posix()])
    if bg_color := args.bg_color:
        params.extend(["--bg-color", bg_color])
    if fg_color := args.fg_color:
        params.extend(["--fg-color", fg_color])
    if image_width := args.image_width:
        params.extend(["--image-width", image_width])
    if image_height := args.image_height:
        params.extend(["--image-height", image_height])
    return params


def generate_image(
    config: Path,
    output_dir: Path,
    rust_binary: Path,
    common_params: list[str],
    dry_run: bool,
):
    output_path = output_dir / f"{config.stem}.png"
    command = [
        rust_binary.resolve().as_posix(),
        "--configuration-path",
        config.resolve().as_posix(),
        "--output-path",
        output_path.resolve().as_posix(),
        *common_params,
    ]
    if dry_run:
        print(f"Would execute:\n{command}")
        return
    subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=True,
    )


def main():
    args = parse_args()

    common_params = binary_common_params(args)

    for config in args.configuration_dir.glob("*.toml"):
        print(f"Processing {config.resolve().as_posix()} ...")
        try:
            generate_image(config, args.output_dir, args.rust_binary, common_params, args.dry_run)
        except subprocess.CalledProcessError as e:
            msg = [str(e), f"stdout: {e.stdout}", f"stderr: {e.stderr}"]
            print("\n".join(msg), end="")

    print(f"All images saved in {Path(args.output_dir).resolve().as_posix()}")


if __name__ == "__main__":
    main()
