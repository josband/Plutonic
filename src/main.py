from dotenv import load_dotenv
from plutonic import Plutonic
import argparse
import os

# Set up argument parsing
parser = argparse.ArgumentParser()

# Load .env file and get API keys
load_dotenv()
api_key = os.environ['ALPACA_API_KEY']
secret_key = os.environ['ALPACA_API_SECRET']

# Initialize bot
bot = Plutonic(api_key=api_key, secret_key=secret_key)