"""
Plutonic Functionality. Everything needed for automating trades are here.
"""

from alpaca.trading.client import TradingClient
import utils.logger

class Plutonic:
    """
    Plutonic Trading Bot. This is the main interface in which to operate it
    """
    
    def __init__(self, api_key=None, secret_key=None) -> None:
        utils.logger.info('Starting Plutonic Initialization')
        try:
            self.client = TradingClient(api_key, secret_key)
            utils.logger.success('Successfully Connected Trading Client')
        except ValueError:
            utils.logger.error("Could Not Successfully Load API Client")
            exit(1)

        utils.logger.success('Initialization Complete')
