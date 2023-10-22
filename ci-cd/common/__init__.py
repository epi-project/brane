#   INIT  .py
#   by Lut99
#
# Created:
#   02 Oct 2023, 14:52:22
# Last edited:
#   04 Oct 2023, 15:03:06
# Auto updated?
#   Yes
#
# Description:
#   Defines shared helper functions and other utilities across all CI/CD
#   scripts.
#

import os
import subprocess
import sys
import typing


##### GLOBALS #####
# Determines whether `pdebug()` calls print anything.
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

def run_command(cmd: list[str], cwd: typing.Optional[str] = None, env:dict[str, str] = os.environ, print_stdout: bool = False, print_stderr: bool = False) -> int:
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
    try:
        handle = subprocess.Popen(cmd, env=env, cwd=cwd, stdout=subprocess.PIPE if print_stdout else None, stderr=subprocess.PIPE if print_stderr else None)
        stdout, stderr = handle.communicate()

        # Print stdout/stderr if told to do so
        if print_stdout:
            print(f"stdout:\n{'-' * 80}\n{stdout}\n{'-' * 80}\n")
        if print_stderr:
            print(f"stdout:\n{'-' * 80}\n{stderr}\n{'-' * 80}\n")

        # Alright cowboy that's it
        return handle.returncode
    except Exception as e:
        perror(f"Failed to start process: {e}")
        return 1
