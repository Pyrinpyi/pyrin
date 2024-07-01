import asyncio
import unittest
import pyrin

class TestWallet(unittest.IsolatedAsyncioTestCase):

    # async def test_wallet(self):
    #     wallet = pyrin.Wallet()
    #     r = await wallet.connect()
    #     account = await wallet.create_account()
    #
    #     # TODO: wallet.import("as as as") also with private key
    #
    #     await account.init()
    #
    #     print(dir(account))
    #     print(account.receive_address)
    #
    #     self.assertEqual(account.receive_address[:5], "pyrin")
    #     self.assertEqual(account.receive_address[5], ":")
    #     self.assertEqual(len(account.receive_address[6:]), 61)
    #
    #     fee = account.estimate(100000, 1000000)
    #     print("fee", fee)
    #
    #     # while True:
    #     #     b = account.balance()
    #     #     print(b.mature, b.pending, b.outgoing)
    #     #     await asyncio.sleep(1)
    #
    #     # account.balance
    #     # account.send()
    #     # account.estimate_fee()
    #     # account.change_address()
    #     # somehow listen to changes like balance etc events
    #
    #     print("r", r) # TODO:

    # async def test_wallet_import(self):
    #     wallet = pyrin.Wallet()
    #     success = await wallet.connect()
    #     account = await wallet.import_account("afraid taste gown fine special solve gun program thunder raise nest width core silk kidney post surround excuse endless laundry then keen sell fashion")
    #
    #     self.assertTrue(success)
    #     self.assertTrue(account is not None)
    #     self.assertEqual(account.receive_address, "pyrin:qpf5v0na3tygxpfgr3ej2rzje7qflwnnlfa4exe5vjhx7kedcfljwrtcvze7j")

    # async def test_wallet_import(self):
    #     wallet = pyrin.Wallet()
    #     success = await wallet.connect()
    #     account = await wallet.import_account("salute breeze kangaroo sword candy grass zero tent beef happy nice embrace devote venture swift wasp game horror obvious deputy same deny cushion clap")
    #
    #     await account.init()
    #
    #     print(account.receive_address)
    #
    #     b = account.balance()
    #     print(b.mature, b.pending, b.outgoing)
    #
    #     fee = await account.estimate(0.1, 0.2)
    #     print("fee", fee)
    #
    #     result = await account.send("pyrin:qpwx6a66j38gqgxcvc74ts77fkxhdzdunl6uhvdcplp0cgvvwrx86n2zl67u0", 0.1, 0.2)
    #     print("aggregated_utxos", result.aggregated_utxos)
    #     print("aggregated_fees", result.aggregated_fees)
    #     print("number_of_generated_transactions", result.number_of_generated_transactions)
    #     print("final_transaction_amount", result.final_transaction_amount)
    #     print("final_transaction_id", result.final_transaction_id)

    # async def test_wallet_account_change_address(self):
    #     wallet = pyrin.Wallet()
    #     success = await wallet.connect()
    #     account = await wallet.import_account("salute breeze kangaroo sword candy grass zero tent beef happy nice embrace devote venture swift wasp game horror obvious deputy same deny cushion clap")
    #
    #     self.assertEqual(account.change_address(), "pyrin:qz75jx7g5kg7ceaml22e6sv07y4yafeesrl2mchs8sadg644d0shghguyy8yq")

    async def testasd(self):
        wallet = pyrin.Wallet()
        success = await wallet.connect()
        account = await wallet.import_account("salute breeze kangaroo sword candy grass zero tent beef happy nice embrace devote venture swift wasp game horror obvious deputy same deny cushion clap")

        await account.init()

        def dda_score(data):
            print(f"dda_score: {data}")


        def sync_state(is_synced):
            print(f"sync-state: {is_synced}")

        def balance(b):
            print("balance:", b.mature, b.pending, b.outgoing)

        def disconnect(url):
            print(f"Disconnected from {url}")

        # pyrin.call_with_callback(my_callback, {"key": "value"})

        await account.listen("dda-score", dda_score)
        await account.listen("balance", balance)
        await account.listen("sync-state", sync_state)
        await account.listen("disconnect", disconnect)

        # await account.send("pyrin:qq5ckhmt8l3qt96sm2wpcnewv0ezkr09677w6kpe32u5ds4ztjpl7q4qks6uk", 0.1, 0.2)

        while True:
            b = account.balance()
            print(account.receive_address, b.mature, b.pending, b.outgoing)
            await asyncio.sleep(1)

if __name__ == "__main__":
    unittest.main()