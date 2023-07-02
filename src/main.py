"""
Main file where Plutonic is initialized and ran.
"""

import os
import argparse
import dotenv
from plutonic import Plutonic

# Set up argument parsing
parser = argparse.ArgumentParser()

# Load .env file and get API keys
dotenv.load_dotenv()
api_key = os.environ['ALPACA_API_KEY']
secret_key = os.environ['ALPACA_API_SECRET']

# Initialize bot
bot = Plutonic(api_key=api_key, secret_key=secret_key)
