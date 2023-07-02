"""
Functionailty needed for logging
"""

PURPLE = '\033[95m'
CYAN = '\033[96m'
DARKCYAN = '\033[36m'
BLUE = '\033[94m'
GREEN = '\033[92m'
YELLOW = '\033[93m'
RED = '\033[91m'
BOLD = '\033[1m'
UNDERLINE = '\033[4m'
END = '\033[0m' # back to default

def info(*args) -> None:
    """
    For loggging normal information
    """
    print(f'[{BOLD}INFO{END}]:', *args)

def success(*args) -> None:
    """
    For loggging successful operations
    """
    print(f'[{GREEN + BOLD}SUCCESS{END}]:', *args)

def warning(*args) -> None:
    """
    For loggging any warnings
    """
    print(f'[{YELLOW + BOLD}WARNING{END}]:', *args)

def error(*args) -> None:
    """
    For loggging failures
    """
    print(f'[{RED + BOLD}ERROR{END}]:', *args)
