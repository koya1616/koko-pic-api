# Koko Pic API

Rust、Axum、およびPostgreSQLを使用して構築された画像共有プラットフォームのためのRESTful APIです。

## 概要

Koko Pic APIは、画像共有アプリケーションのためのモダンで高性能なバックエンドサービスです。Rustの堅牢なエコシステムを使用して構築されており、安全なユーザー認証、データ管理を提供し、API開発のベストプラクティスに従っています。

### 特徴

- RESTful API設計
- JWTベースの認証
- PostgreSQLデータベース統合
- Dockerコンテナ化
- 自動データベースマイグレーション
- tracingによる構造化ロギング
- 包括的なエラーハンドリング

## 前提条件

始める前に、以下の要件を満たしていることを確認してください：

- DockerとDocker Compose
- Rust（オプション、プロジェクトはコンテナ内で実行されます）

## セットアップとインストール

1. リポジトリをクローンします：
```bash
git clone https://github.com/kuuuuya/koko-pic-api.git
cd koko-pic-api
```

2. Docker Composeを使用してサービスを起動します：
```bash
make build
make up
```

APIは `http://0.0.0.0:8000` でアクセスできます。

## プロジェクト構造

```
koko-pic-api/
├── Cargo.toml          # Rustの依存関係とプロジェクトメタデータ
├── Dockerfile          # プロダクション用Dockerfile
├── Dockerfile.dev      # 開発用Dockerfile
├── docker-compose.yml  # Docker Compose設定
├── Makefile            # 共通のコマンドとワークフロー
├── migrations/         # SQLマイグレーションファイル
├── openapi.yaml        # OpenAPI仕様
├── src/
│   ├── app.rs          # アプリケーションルータ設定
│   ├── db/             # データベース接続とユーティリティ
│   ├── domains/        # ドメインごとに整理されたビジネスロジック
│   ├── lib.rs          # モジュール宣言
│   ├── main.rs         # アプリケーションのエントリーポイント
│   ├── state.rs        # アプリケーション状態管理
│   └── utils.rs        # ユーティリティ関数
└── ...
```

## 利用可能なコマンド

プロジェクトには便利なコマンドが含まれるMakefileがあります：

- `make build` - 開発用コンテナイメージをビルド
- `make up` - APIとPostgresコンテナをフォアグラウンドで起動
- `make down` - コンテナを停止してコンポーズスタックを削除
- `make logs` - スタックのコンテナログを追跡
- `make shell` - `app`コンテナ内にシェルを開く
- `make restart` - 再構築付きでコンポーズサービスを再起動
- `make ps` - 実行中のコンテナを表示
- `make check` - コンテナ内でフォーマット、リンター、テストを実行
- `make build-prod` - プロダクションイメージをビルド
- `make up-prod` - プロダクションサービスを起動
- `make down-prod` - プロダクションサービスを停止
- `make push-prod` - プロダクションイメージをレジストリにビルドしてプッシュ

## APIエンドポイント

APIは以下のエンドポイントで構成されています：

- `GET /` - "Hello, World!"を返すヘルスチェックエンドポイント
- `POST /api/v1/users` - 新しいユーザーアカウントを作成
- `POST /api/v1/login` - ユーザーを認証してJWTトークンを返す

詳細なAPI仕様については、[OpenAPI仕様](./openapi.yaml)を参照してください。

## 開発

### テストの実行

コンテナ内でテストを実行するには：
```bash
make shell
cargo test
```

### データベースマイグレーション

マイグレーションは起動時に自動的に処理されますが、手動で実行することもできます：
```bash
# コンテナ内で
sqlx migrate run
```

### コードフォーマットとリンティング

コードをフォーマットおよびリンティングするには：
```bash
make check
```

## 環境変数

アプリケーションは以下の環境変数を使用します：

- `DATABASE_URL` - PostgreSQLデータベース接続文字列
- `JWT_SECRET` - JWTトークン署名のシークレットキー
- `PORT` - APIサーバーのポート番号

これらは `docker-compose.yml` ファイルで設定されています。

## 貢献

1. リポジトリをフォーク
2. 機能ブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更を加える
4. フォーマッタとリンターを実行 (`make check`)
5. 変更をコミット (`git commit -m 'Add amazing feature'`)
6. ブランチにプッシュ (`git push origin feature/amazing-feature`)
7. プルリクエストを開く

## ライセンス

このプロジェクトはMITライセンスの下でライセンスされています - 詳細については[LICENSE](LICENSE)ファイルを参照してください。