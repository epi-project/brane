#!/usr/bin/env python3
# CI CD.py
#   by Lut99
#
# Created:
#   02 Oct 2023, 14:57:26
# Last edited:
#   03 Oct 2023, 10:37:03
# Auto updated?
#   Yes
#
# Description:
#   Main entrypoint that provides nested access to all CI/CD scripts.
#

import argparse
import typing

import common
from audit import code_quality, dependencies
from ci import unit_tests


##### ENTRYPOINT #####
def main(category: str, task: typing.Optional[str], args: argparse.Namespace) -> int:
    """
        Entrypoint to the script.

        # Arguments
        - `category`: The user-specified category (i.e., first subcommand) to execute.
        - `task`: The user-specified task within the chosen category (i.e., second subcommand) to execute. If `None`, then all tasks within the chosen category will be executed.

        # Returns
        The intended exit code of this script.
    """

    # Match on the subcommand
    if category == "audit":
        if task is None or task == "dependencies":
            return dependencies.run(args)
        if task is None or task == "code_quality":
            return code_quality.run(args)
        if task is not None and task != "code_quality" and task != "dependencies":
            common.perror(f"Unknown audit task '{task}'")
            return 1

    elif category == "ci":
        if task is None or task == "unit_tests":
            return unit_tests.run(args)
        if task is not None and task != "unit_tests":
            common.perror(f"Unknown ci task '{task}'")
            return 1

    else:
        common.perror(f"Unknown category '{category}'")
        return 1

    # Done!
    return 0



# Actual entrypoint
if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--debug", action="store_true", help="If given, enables additional debug prints.")
    parser.add_argument("--os", required=True, choices=["windows", "macos", "ubuntu", "arch"], help="Determines the installation stuff for the target OS.")
    parser.add_argument("--repo", required=True, help="Sets the path of the repository which we will perform CI/CD on.")
    parser.add_argument("--refresh-mirrors", action="store_true", help="Refreshes the mirrors before pulling with apt-get and all that.")
    subparsers = parser.add_subparsers(dest="category", required=True)

    # Add the audit subcommand etc
    audit_parser = subparsers.add_parser("audit")
    audit_subparsers = audit_parser.add_subparsers(dest="task")
    audit_subparsers.add_parser("code_quality", parents=[code_quality.parser], add_help=False)
    audit_subparsers.add_parser("dependencies", parents=[dependencies.parser], add_help=False)

    # Add the ci subcommand etc
    audit_parser = subparsers.add_parser("ci")
    audit_subparsers = audit_parser.add_subparsers(dest="task")
    audit_subparsers.add_parser("unit_tests", parents=[unit_tests.parser], add_help=False)

    # Alright parse everything
    args = parser.parse_args()
    common.DEBUG = args.debug

    # Run the main
    exit(main(args.category, args.task, args))
