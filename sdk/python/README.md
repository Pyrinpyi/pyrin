

#### Pyrin SDK Python Bindings 

Built using [PyO3](https://github.com/PyO3/pyo3)

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
```