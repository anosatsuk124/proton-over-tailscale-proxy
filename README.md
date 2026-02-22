# ProtonVPN over Tailscale Proxy

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-18+-61DAFB.svg)](https://reactjs.org/)

Tailscaleネットワークを通じてProtonVPNに接続するためのプロキシシステムです。セキュアなVPN出口ノードをTailscaleネットワーク内に簡単にデプロイできます。

## 概要

このプロジェクトは、Dockerコンテナ内でProtonVPNとTailscaleを組み合わせ、Tailscaleネットワークを通じて利用可能なセキュアなVPN出口ノードを提供します。Webダッシュボードから接続を管理できます。

### 主な機能

- **ProtonVPN統合**: WireGuardプロトコルを使用した安全なVPN接続
- **Tailscale出口ノード**: Tailscaleネットワーク内でVPN出口として機能
- **Webダッシュボード**: リアルタイムで接続状態を監視・制御
- **REST API**: プログラムによるVPN制御
- **Docker対応**: 簡単なデプロイとスケーリング

## アーキテクチャ

```
┌─────────────────┐
│   Webブラウザ   │
│   (React UI)    │
└────────┬────────┘
         │
         │ HTTP/WS
         │
┌────────▼────────┐
│  Rust Backend   │
│   (Axum API)    │
└────────┬────────┘
         │
         │ Docker API
         │
┌────────▼──────────────────────────┐
│        Dockerホスト               │
│  ┌─────────────────────────────┐ │
│  │  VPNコンテナ                │ │
│  │  ┌──────────┐  ┌──────────┐ │ │
│  │  │ProtonVPN │──│Tailscale │ │ │
│  │  │(出口)    │  │(出口ノード)│ │ │
│  │  └──────────┘  └──────────┘ │ │
│  └─────────────────────────────┘ │
└───────────────────────────────────┘
```

## クイックスタート

### 必要条件

- Docker 24.0+
- Docker Compose 2.0+
- Rust 1.75+ (開発時)
- Node.js 18+ (開発時)

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

## 設定

### 環境変数

| 変数 | 説明 | 必須 |
|------|------|------|
| `PROTONVPN_USERNAME` | ProtonVPNユーザー名 | ✅ |
| `PROTONVPN_PASSWORD` | ProtonVPNパスワード | ✅ |
| `TAILSCALE_AUTH_KEY` | Tailscale認証キー | ✅ |
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

詳細は [docs/CONFIGURATION.md](./docs/CONFIGURATION.md) のトラブルシューティングセクションを参照してください。

## 貢献

貢献を歓迎します！詳細は [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md) を参照してください。

## ライセンス

MIT License - 詳細は [LICENSE](./LICENSE) ファイルを参照してください。

## 関連リンク

- [ProtonVPN](https://protonvpn.com/)
- [Tailscale](https://tailscale.com/)
- [WireGuard](https://www.wireguard.com/)

---

[English README](./README_EN.md) is also available.
