# crontab-modoki

crontab モドキ。

## 開発

```sh
cat << EOF > .conf
00 * * * * rustc --version
EOF
```

Run

```sh
cargo run ./.conf
```

## ビルド

```sh
cargo build
```

or

```sh
cargo build --release
```

## クロスコンパイルによるビルド

```sh
cargo install cross
```

### x86_64-pc-windows-gnu

```sh
cross build --target x86_64-pc-windows-gnu
```

## x86_64-unknown-linux-gnu

```sh
cross build --target x86_64-unknown-linux-gnu
```

## Memo

- Linux: systemd
- macOS: launchd
- Windows: ?
