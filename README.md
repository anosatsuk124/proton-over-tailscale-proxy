# ProtonVPN Tailscale Exit Node

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-18+-61DAFB.svg)](https://reactjs.org/)

Tailscaleネットワーク内でProtonVPNを出口ノード（Exit Node）として提供するシステムです。Tailscaleクライアントから簡単にアクセスできる、セキュアなVPN出口ノードをデプロイできます。

## 概要

このプロジェクトは、Dockerコンテナ内でProtonVPNとTailscaleを組み合わせ、Tailscaleネットワーク内のデバイスから利用できるVPN出口ノードを提供します。Webダッシュボードから接続を管理でき、Tailscaleのexit node機能を通じてネットワーク全体のトラフィックをVPN経由でルーティングできます。

### 主な機能

- **ProtonVPN統合**: WireGuardプロトコルを使用した安全なVPN接続
- **Tailscale出口ノード**: Tailscaleネットワーク内で正式な出口ノードとして機能
- **全デバイス対応**: クライアント側でSOCKSプロキシ設定が不要
- **Webダッシュボード**: リアルタイムで接続状態を監視・制御
- **REST API**: プログラムによるVPN制御
- **Docker対応**: 簡単なデプロイとスケーリング
- **自動NAT/マスカレード**: クライアントトラフィックの自動変換

## アーキテクチャ

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Tailscale      │     │  Tailscale      │     │  Tailscale      │
│  Client #1      │     │  Client #2      │     │  Client #3      │
│  (iOS/Android)  │     │  (Windows)      │     │  (Linux/macOS)  │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                       │
         │    Tailscale Mesh     │    VPN (WireGuard)    │
         │         Network       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────▼─────────────┐
                    │    Tailscale Network      │
                    │    (Encrypted Mesh)       │
                    └─────────────┬─────────────┘
                                  │
                    ┌─────────────▼─────────────┐
                    │   Exit Node Container     │
                    │  ┌─────────────────────┐  │
                    │  │   tailscaled        │  │
                    │  │  (Userspace Mode)   │  │
                    │  └──────────┬──────────┘  │
                    │             │             │
                    │  ┌──────────▼──────────┐  │
                    │  │   WireGuard         │  │
                    │  │   (ProtonVPN)       │  │
                    │  └──────────┬──────────┘  │
                    │             │             │
                    │  ┌──────────▼──────────┐  │
                    │  │   NAT/Masquerade    │  │
                    │  │   (iptables)        │  │
                    │  └─────────────────────┘  │
                    └───────────────────────────┘
                                 │
                                 ▼
                    ┌───────────────────────────┐
                    │      ProtonVPN            │
                    │      (Internet)           │
                    └───────────────────────────┘

トラフィックフロー:
クライアント → Tailscale → tailscaled → WireGuard → ProtonVPN → インターネット
```

## クイックスタート

### 必要条件

- Docker 24.0+
- Docker Compose 2.0+
- Rust 1.75+ (開発時)
- Node.js 18+ (開発時)
- Tailscaleアカウント
- ProtonVPNアカウント

### インストール

1. リポジトリのクローン:
```bash
git clone https://github.com/anosatsuk124/proton-over-tailscale-proxy.git
cd proton-over-tailscale-proxy
```

2. 環境変数の設定:
```bash
cp config/.env.example config/.env
# config/.envを編集して認証情報を設定
```

3. Dockerで実行:
```bash
docker compose up -d
```

4. Webダッシュボードにアクセス:
```
http://localhost:3000
```

## Tailscale出口ノードの設定

### 1. Tailscale管理コンソールでの承認

コンテナ起動後、Tailscale管理コンソールで出口ノードとして承認する必要があります:

1. [Tailscale Admin Console](https://login.tailscale.com/admin/machines)にアクセス
2. デプロイしたマシンを見つける（`TAILSCALE_HOSTNAME`で設定した名前）
3. マシン名の横にある「**...**」（メニュー）をクリック
4. 「**Edit route settings...**」を選択
5. 「**Use as exit node**」を有効化
6. 「**Save**」をクリック

### 2. クライアントからの接続

承認後、Tailscaleネットワーク内の任意のデバイスからこの出口ノードを使用できます:

**コマンドライン:**
```bash
# Linux/macOS
tailscale up --exit-node=proton-vpn-exit

# 出口ノードの使用を停止
tailscale up --exit-node=
```

**GUIアプリ:**
- **iOS/Android**: Tailscaleアプリ → 出口ノード → マシンを選択
- **Windows/macOS**: メニューバー/タスクトレイ → Tailscale → Exit Node → マシンを選択

**検証:**
```bash
# 出口ノード経由でインターネットに接続されているか確認
curl https://ipinfo.io
```

## 設定

### 環境変数

| 変数 | 説明 | 必須 |
|------|------|------|
| `PROTONVPN_USERNAME` | ProtonVPNユーザー名 | ✅ |
| `PROTONVPN_PASSWORD` | ProtonVPNパスワード | ✅ |
| `TAILSCALE_AUTH_KEY` | Tailscale認証キー | ✅ |
| `TAILSCALE_HOSTNAME` | Tailscaleネットワーク内のホスト名 | デフォルト: proton-vpn-exit |
| `TAILSCALE_ADVERTISE_EXIT_NODE` | 出口ノードとしてアドバタイズ | デフォルト: true |
| `API_PORT` | APIサーバーポート | デフォルト: 8080 |
| `FRONTEND_PORT` | フロントエンドポート | デフォルト: 3000 |

詳細な設定オプションは [docs/CONFIGURATION.md](./docs/CONFIGURATION.md) を参照してください。

## API使用例

### 接続状態の確認

```bash
curl http://localhost:8080/status
```

### VPN接続の開始

```bash
curl -X POST http://localhost:8080/connect \
  -H "Content-Type: application/json" \
  -d '{"server": "JP-FREE#1", "protocol": "wireguard"}'
```

### VPN接続の停止

```bash
curl -X POST http://localhost:8080/disconnect
```

### 出口ノード状態の確認

```bash
curl http://localhost:8080/exit-node
```

### 出口ノードの有効化/無効化

```bash
# 有効化
curl -X POST http://localhost:8080/exit-node \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'

# 無効化
curl -X POST http://localhost:8080/exit-node \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'
```

## ドキュメント

- [アーキテクチャ](./docs/ARCHITECTURE.md) - システム設計と技術的決定
- [開発ガイド](./docs/DEVELOPMENT.md) - 開発環境のセットアップと貢献ガイドライン
- [APIドキュメント](./docs/API.md) - 完全なAPIエンドポイント仕様
- [デプロイメント](./docs/DEPLOYMENT.md) - 本番環境へのデプロイ手順
- [設定ガイド](./docs/CONFIGURATION.md) - 設定オプションとカスタマイズ

## トラブルシューティング

### コンテナが起動しない

```bash
# ログを確認
docker compose logs -f vpn

# 設定を検証
docker compose config
```

### VPN接続が確立しない

1. ProtonVPN認証情報を確認
2. ポート転送設定を確認（WireGuardはUDP 51820を使用）
3. ファイアウォールルールを確認

### 出口ノードが機能しない

1. **Tailscale管理コンソールで承認されているか確認**
   - [Admin Console](https://login.tailscale.com/admin/machines)で「Use as exit node」が有効になっているか確認

2. **クライアント側の設定を確認**
   ```bash
   # 現在の出口ノード設定を確認
   tailscale status
   
   # 出口ノードが正しく設定されているか確認
   tailscale up --exit-node=proton-vpn-exit
   ```

3. **NAT/マスカレードが有効か確認**
   ```bash
   # コンテナ内で確認
   docker exec proton-vpn iptables -t nat -L
   
   # POSTROUTINGチェーンにMASQUERADEルールがあるか確認
   ```

4. **Tailscaleが出口ノードとしてアドバタイズしているか確認**
   ```bash
   docker exec proton-vpn tailscale status
   
   # 出力に「offers exit node」が含まれているか確認
   ```

5. **ルーティングの確認**
   ```bash
   # コンテナ内でデフォルトルートを確認
   docker exec proton-vpn ip route
   
   # WireGuardインターフェース（wg0）がデフォルトルートになっているか確認
   ```

### クライアントが出口ノードに接続できない

1. **Tailscaleネットワークに接続されているか確認**
   ```bash
   tailscale status
   ```

2. **出口ノードの名前が正しいか確認**
   ```bash
   # 利用可能な出口ノードを一覧表示
   tailscale exit-node list
   ```

3. **ACL設定を確認**
   - Tailscale ACLで出口ノードへのアクセスが許可されているか確認
   - [ACL Documentation](https://tailscale.com/kb/1018/acls)

詳細は [docs/CONFIGURATION.md](./docs/CONFIGURATION.md) のトラブルシューティングセクションを参照してください。

## 貢献

貢献を歓迎します！詳細は [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md) を参照してください。

## ライセンス

MIT License - 詳細は [LICENSE](./LICENSE) ファイルを参照してください。

## 関連リンク

- [ProtonVPN](https://protonvpn.com/)
- [Tailscale](https://tailscale.com/)
- [Tailscale Exit Nodes](https://tailscale.com/kb/1103/exit-nodes)
- [WireGuard](https://www.wireguard.com/)

---

[English README](./README_EN.md) is also available.
