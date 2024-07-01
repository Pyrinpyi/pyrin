import unittest
import pyrin

class TestBip32(unittest.TestCase):

    def test_mnemonic(self):
        bip32 = pyrin.Bip32() # TODO: Convert to static class ?

        # 24 words
        words = bip32.generate_mnemonic().split()

        self.assertIs(len(words), 24)
        self.assertNotEqual(bip32.generate_mnemonic(), bip32.generate_mnemonic())

        # 12 words
        words = bip32.generate_short_mnemonic().split()

        self.assertIs(len(words), 12)
        self.assertNotEqual(bip32.generate_short_mnemonic(), bip32.generate_short_mnemonic())

class TestAsyncBip32(unittest.IsolatedAsyncioTestCase):
    async def test_asd(self):
        bip32 = pyrin.Bip32()

        len = await bip32.generate_mnemonicasd()
        print("len", len)

if __name__ == "__main__":
    unittest.main()