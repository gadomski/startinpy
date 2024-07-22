# Installation

## pip

```console
$ pip install startinpy
```

<!-- :::{attention}
The building of Linux wheels is not really working at the moment, so you'll have to compile the code yourself, see below.
For macOS and Windows, pip will work fine.
::: -->

## If you want to compile it yourself

1. get the code: <https://github.com/hugoledoux/startinpy>
2. install latest [Rust](https://www.rust-lang.org/)
3. install [maturin](https://github.com/PyO3/maturin)
4. `maturin build --release`
5. `cd ./target/wheels/`
6. `pip install [name-wheel].whl` will install it to your local Python

## Development (to debug the code)

1. get the code: <https://github.com/hugoledoux/startinpy>
2. install latest [Rust](https://www.rust-lang.org/)
3. install [maturin](https://github.com/PyO3/maturin)
4. compile the rust code and build the Python bindings (in debug mode, thus slow):

```console
$ maturin develop
```

5. move to another folder, and this shouldn't return any error:

```console
$ python
$ import startinpy
```
