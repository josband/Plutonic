class Color:
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
    print(f'[{Color.BOLD}INFO{Color.END}]:', *args)

def success(*args) -> None:
    print(f'[{Color.GREEN + Color.BOLD}SUCCESS{Color.END}]:', *args)

def warning(*args) -> None:
    print(f'[{Color.YELLOW + Color.BOLD}WARNING{Color.END}]:', *args)

def error(*args) -> None:
    print(f'[{Color.RED + Color.BOLD}ERROR{Color.END}]:', *args)