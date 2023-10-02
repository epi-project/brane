# CODE QUALITY.py
#   by Lut99
#
# Created:
#   02 Oct 2023, 16:51:52
# Last edited:
#   02 Oct 2023, 17:46:38
# Auto updated?
#   Yes
#
# Description:
#   Performs the setup & execution of a `clippy` run in either GitHub
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
        raise ValueError(f"Dependency analysis is not implemented for non-Ubuntu operating systems such as '{os_id}'")
    elif os_id == "macos":
        raise ValueError(f"Dependency analysis is not implemented for non-Ubuntu operating systems such as '{os_id}'")
    elif os_id == "ubuntu":
        return setup_ubuntu(refresh_mirrors)
    elif os_id == "arch":
        raise ValueError(f"Dependency analysis is not implemented for non-Ubuntu operating systems such as '{os_id}'")
    else:
        raise ValueError(f"Unsupported OS string '{os_id}'")

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
        if code := common.run_command([ "sed", "-i", "s/htt[p|ps]:\/\/archive.ubuntu.com\/ubuntu\//mirror:\/\/mirrors.ubuntu.com\/mirrors.txt/g", "/etc/apt/sources.list" ]): return code

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

    # Prepare the environment with cargo stuff
    env = os.environ.copy()
    env["PATH"] = f"{env['PATH']}:/root/.cargo/bin"

    # Run the cargo audit
    if code := common.run_command([ "cargo", "clippy", "--all-targets", "--all-features", "--", "-D", "warnings", "--allow", "clippy::manual_range_contains" ], cwd=args.repo, env=env):
        common.perror(f"Cargo audit failed with return code {code} (see output above)")
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
