#!/usr/bin/env python3
# DEPENDENCIES.py
#   by Lut99
#
# Created:
#   26 Apr 2023, 15:16:49
# Last edited:
#   02 Oct 2023, 13:56:13
# Auto updated?
#   Yes
#
# Description:
#   Python script for implementing the `audit` GitHub action. Essentially
#   just calls `cargo audit`, after installing the target stuff.
#

import argparse
import os
import subprocess
import sys
import typing


##### GLOBALS #####
# Defines whether we are in debug mode or not
DEBUG: bool = False





##### HELPER FUNCTIONS #####
def supports_color() -> bool:
    """
        Returns True if the running system's terminal supports color, and False
        otherwise.

        From: https://stackoverflow.com/a/22254892
    """
    plat = sys.platform
    supported_platform = plat != 'Pocket PC' and (plat != 'win32' or
                                                  'ANSICON' in os.environ)
    # isatty is not always implemented, #6223.
    is_a_tty = hasattr(sys.stdout, 'isatty') and sys.stdout.isatty()
    return supported_platform and is_a_tty

def pdebug(text: str):
    """
        Prints some output to stdout as if it was a debug string.
    """

    # Only print if debugging
    if DEBUG:
        # Determine is we're on a colour terminal or na
        use_colour: bool = supports_color()

        # Resolve the colours
        start = "\033[90;1m" if use_colour else ""
        end   = "\033[0m" if use_colour else ""

        # Print the text
        print(f"{start}[DEBUG] {text}{end}")
def perror(text: str):
    """
        Prints some output to stdout as if it was a debug string.
    """

    # Resolve the colours
    use_colour: bool = supports_color()
    start = "\033[91;1m" if use_colour else ""
    bold  = "\033[1m" if use_colour else ""
    end   = "\033[0m" if use_colour else ""

    # Print the text
    print(f"{start}[ERROR]{end} {bold}{text}{end}")

def run_command(cmd: list[str], cwd: typing.Optional[str] = None, env:dict[str, str] = os.environ) -> int:
    """
        Runs the given command as a subprocess, with some nice printing in advance.

        Arguments:
        - `cmd`: The command (already separated in the arguments) to execute.
        
        Returns:
        The return code of the command.
    """

    # Determine the printing colours
    use_colour = supports_color()
    start = "\033[1m" if use_colour else ""
    end   = "\033[0m" if use_colour else ""

    # Print it
    print(f"{start} >", end="")
    for c in cmd:
        c = c.replace("\\", "\\\\").replace("\"", "\\\"")
        if ' ' in c:
            print(f" \"{c}\"", end="")
        else:
            print(f" {c}", end="")
    print(f"{end}")

    # Run it as a subprocess
    handle = subprocess.Popen(cmd, env=env, cwd=cwd)
    handle.communicate()
    return handle.returncode





##### SETUP FUNCTIONS #####
def setup(os_id: str) -> int:
    """
        Function that initializes the environment.

        Assumes that we are running in one of the following containers, based
        on the given `os` string:
        - `windows` -> 
        - `macos` -> 
        - `ubuntu` -> ubuntu:22.04
        - `arch` -> 

        Returns 0 on success, or else some error code.
    """

    # Switch on the given OS for proper setup
    if os_id == "windows":
        pass
    elif os_id == "macos":
        pass
    elif os_id == "ubuntu":
        return setup_ubuntu()
    elif os_id == "arch":
        pass
    else:
        raise ValueError(f"Unsupported OS string `{os}`")

def setup_ubuntu() -> int:
    """
        Function that initializes the environment within the Ubuntu container
        (ubuntu:22.04).

        Returns 0 on success, or else some error code.
    """

    pdebug("Initializing Ubuntu environment...")

    # Fix the mirrors
    if code := run_command([ "apt-get", "update" ]): return code
    if code := run_command([ "apt-get", "install", "-y", "ca-certificates" ]): return code
    if code := run_command([ "sed", "-i", "s/htt[p|ps]:\/\/archive.ubuntu.com\/ubuntu\//mirror:\/\/mirrors.ubuntu.com\/mirrors.txt/g", "/etc/apt/sources.list" ]): return code

    # Install the build stuff
    if code := run_command([ "apt-get", "update" ]): return code
    # if code := run_command([ "apt-get", "install", "-y", "curl", "git", "gcc", "g++", "cmake", "pkg-config", "libssl-dev" ]): return code
    if code := run_command([ "apt-get", "install", "-y", "curl", "gcc", "g++", "cmake", "pkg-config", "libssl-dev" ]): return code

    # Install Rust
    if code := run_command([ "bash", "-c", "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile default -y" ]): return code
    if code := run_command([ "/root/.cargo/bin/cargo", "install", "cargo-audit" ]): return code

    # Done
    pdebug("Done initializing environment")
    return 0





##### ENTRYPOINT #####
def main(os_id: str) -> int:
    """
        Entrypoint to the script. Argument parsing is abstracted away by the
        caller.

        # Arguments
        - `os`: The OS identifier to use to determine what to install and how.
    """

    # Prepare the environment first
    if code := setup(os_id):
        perror("Failed to prepare environment (see output above)")
        return code

    # Prepare the environment with cargo stuff
    env = os.environ.copy()
    env["PATH"] = f"{env['PATH']}:/root/.cargo/bin"

    # Run the cargo audit
    if code := run_command([ "cargo", "audit" ], cwd="/brane", env=env):
        perror(f"Cargo audit failed with return code {code} (see output above)")
        return code

    # Done!
    return 0


# Actual entrypoint
if __name__ == "__main__":
    # Let's define the arguments
    parser = argparse.ArgumentParser()
    parser.add_argument("OS", choices=["windows", "macos", "ubuntu", "arch"], help="Determines the installation stuff for the target OS.")
    parser.add_argument("REPOSITORY", help="The path to the repository itself.")
    parser.add_argument("--debug", action="store_true", help="If given, enables additional debug prints.")

    # Let's parse the arguments
    args = parser.parse_args()

    # Set some stuff globally
    DEBUG = args.debug

    # Call main
    exit(main(args.OS))
