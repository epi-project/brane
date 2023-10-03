#!/usr/bin/env python3
# UNIT TESTS.py
#   by Lut99
#
# Created:
#   02 Oct 2023, 16:51:52
# Last edited:
#   03 Oct 2023, 16:00:23
# Auto updated?
#   Yes
#
# Description:
#   Performs the setup & execution of a `cargo test` run in either GitHub
#   Actions or a local container.
#

import argparse
import os

import common


#### SETUP FUNCTIONS #####
def setup(os_id: str, refresh_mirrors: bool) -> int:
    """
        Function that initializes the environment.

        Assumes that we are running in one of the following containers, based
        on the given `os_id` string:
        - `windows` -> 
        - `macos` -> 
        - `ubuntu` -> ubuntu:22.04
        - `arch` -> 

        Returns 0 on success, or else some error code.
    """

    # Switch on the given OS for proper setup
    if os_id == "windows":
        return setup_windows()
    elif os_id == "macos":
        raise ValueError(f"Unit testing is not implemented for non-Ubuntu operating systems such as '{os_id}'")
    elif os_id == "ubuntu":
        return setup_ubuntu(refresh_mirrors)
    elif os_id == "arch":
        raise ValueError(f"Unit testing is not implemented for non-Ubuntu operating systems such as '{os_id}'")
    else:
        raise ValueError(f"Unsupported OS string '{os_id}'")

def setup_windows() -> int:
    """
        Function that initializes the environment within the Windows container
        (Windows 10 20H2).

        Returns 0 on success, or else some error code.
    """

    common.pdebug("Initializing Windows environment...")

    # Install the C++ stuff
    if code := common.run_command([ "choco", "install", "-y", "visualcpp-build-tools" ]): return code

    # Download the rustup executable
    if code := common.run_command([ "powershell", "-Command", "Invoke-WebRequest \"https://win.rustup.rs/x86_64\" -OutFile C:\\rustup-init.exe" ]): return code
    # Install Rust
    if code := common.run_command([ "C:\\rustup-init.exe", "--profile", "default", "-y" ]): return code

    # Done
    common.pdebug("Done initializing environment")
    return 0

def setup_ubuntu(refresh_mirrors: bool) -> int:
    """
        Function that initializes the environment within the Ubuntu container
        (ubuntu:22.04).

        Returns 0 on success, or else some error code.
    """

    common.pdebug("Initializing Ubuntu environment...")

    # Fix the mirrors
    if refresh_mirrors:
        if code := common.run_command([ "apt-get", "update" ]): return code
        if code := common.run_command([ "apt-get", "install", "-y", "ca-certificates" ]): return code
        if code := common.run_command([ "sed", "-i", "s/htt[p|ps]:\\/\\/archive.ubuntu.com\\/ubuntu\\//mirror:\\/\\/mirrors.ubuntu.com\\/mirrors.txt/g", "/etc/apt/sources.list" ]): return code

    # Install the build stuff
    if code := common.run_command([ "apt-get", "update" ]): return code
    if code := common.run_command([ "apt-get", "install", "-y", "curl", "gcc", "g++", "cmake", "pkg-config", "libssl-dev" ]): return code

    # Install Rust
    if code := common.run_command([ "bash", "-c", "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile default -y" ]): return code

    # Done
    common.pdebug("Done initializing environment")
    return 0





##### ENTRYPOINT #####
# Let's define the arguments the main parser needs to know about
parser = argparse.ArgumentParser()



def run(args: argparse.Namespace) -> int:
    """
        Entrypoint to the script. Argument parsing is abstracted away by the
        caller.

        # Arguments
        - `args.os`: The OS identifier to use to determine what to install and how.
        - `args.repo`: The path to the Brane repository to audit.
        - `args.refresh_mirrors`: Whether to pull some tricks to refresh the mirrors of apt first or not.

        # Returns
        The exit code which the script as whole should return.
    """

    # Prepare the environment first
    if code := setup(args.os, args.refresh_mirrors):
        common.perror("Failed to prepare environment (see output above)")
        return code

    # # Prepare the environment with cargo stuff
    # env = os.environ.copy()
    # if args.os == "windows":
    #     env["PATH"] = f"{env['PATH']};/root/.cargo/bin"
    # else:
    #     env["PATH"] = f"{env['PATH']}:/root/.cargo/bin"

    # Run the cargo audit
    # if code := common.run_command([ "cargo", "test", "--all-targets", "--all-features" ], cwd=args.repo, env=env):
    if code := common.run_command([ "cargo", "test", "--all-targets", "--all-features" ], cwd=args.repo):
        common.perror(f"Cargo test failed with return code {code} (see output above)")
        return code

    # Done!
    return 0



# Actual entrypoint
if __name__ == "__main__":
    # Add the main arguments to our parser
    parser.add_argument("--debug", action="store_true", help="If given, enables additional debug prints.")
    parser.add_argument("--os", required=True, choices=["windows", "macos", "ubuntu", "arch"], help="Determines the installation stuff for the target OS.")
    parser.add_argument("--repo", required=True, help="Sets the path of the repository which we will perform CI/CD on.")
    parser.add_argument("--refresh-mirrors", action="store_true", help="Refreshes the mirrors before pulling with apt-get and all that.")

    # Parse the arguments
    args = parser.parse_args()
    common.DEBUG = args.debug

    # Call main
    exit(run(args))
