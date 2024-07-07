

#### Pyrin SDK Python Bindings 
[SDK Documentation](https://docs.pyrin.network/7.-sdk/python/getting-started)

Built using [PyO3](https://github.com/PyO3/pyo3)


#### Installation

```bash
pip install pyrin-network
```

#### Running node with wRPC
```bash
pyrin --utxoindex --rpclisten-borsh
```

#### Usage

```python
import pyrin

rpc = pyrin.RPC()
await rpc.connect()
balance = await rpc.get_balance_by_address("pyrin:qzn54t6vpasykvudztupcpwn2gelxf8y9p84szksr73me39mzf69uaalnymtx")
print("balance:", balance)
```

```python
import pyrin

wallet = pyrin.Wallet()
r = await wallet.connect()
account = await wallet.create_account()
```


#### Local Development

```bash 

#### Dependencies

```bash
pip install maturin
```

#### Build

```bash
cargo build

maturin build
pip install <compiled whl file>
```

#### Development

```bash
conda create -n sdk-develop python=3.8
conda activate sdk-develop

cd sdk/python
maturin develop
```

#### Test

```bash
python -m unittest tests/test_wallet.py
python -m unittest tests/test_bip32.py
python -m unittest tests/test_rpc.py
python -m unittest tests/test_lib.py
```