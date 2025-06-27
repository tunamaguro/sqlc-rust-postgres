# Struct-Based API Design and Implementation

## 概要

本文書では、sqlc-rust-postgresプラグインにおける構造体ベースAPI（Struct-Based API）の設計、実装戦略、および技術的判断の根拠について詳述する。この機能は、現在の関数ベースAPIの課題を解決しつつ、ゼロコスト抽象化を実現する新しいAPIパターンを提供する。

## 背景と問題意識

### 現在の課題

現在のsqlc-rust-postgresプラグインが生成するAPIには、以下の重要な問題がある：

#### 1. 関数引数の肥大化問題

SQLクエリのパラメータが増加すると、生成される関数の引数が過度に多くなる：

```rust
// 問題のある例：引数が多すぎて可読性・保守性が低い
pub async fn update_user_profile(
    client: &Client,
    user_id: i64,
    name: &str,
    email: &str,
    phone: &str,
    address: &str,
    birth_date: Option<chrono::NaiveDate>,
    is_verified: bool,
    updated_at: chrono::DateTime<chrono::Utc>,
) -> Result<UpdateUserProfileRow, Error>
```

#### 2. パラメータ順序の誤用リスク

引数の順序は型システムでは検証されないため、SQLクエリの変更により引数順序が変わっても、型が一致していればコンパイルエラーにならない：

```rust
// SQLクエリ変更前：WHERE user_id = $1 AND status = $2
get_user_by_status(client, user_id, status).await?;

// SQLクエリ変更後：WHERE status = $1 AND user_id = $2  
// ↓ 引数順序が入れ替わったが、型は同じなのでコンパイルエラーにならない
get_user_by_status(client, user_id, status).await?; // バグ！
```

#### 3. 型安全性の欠如

関数の引数リストでは、どの値がどのSQLパラメータに対応するかが位置に依存するため、リファクタリング時の安全性が低い。

### 解決すべき要求仕様

上記課題を踏まえ、以下の要求仕様を満たすAPIを設計する：

1. **型安全性の向上**: パラメータの設定漏れや重複設定をコンパイル時に検出
2. **可読性の向上**: どの値がどのパラメータに対応するかが明確
3. **保守性の向上**: SQLクエリの変更に対してより堅牢
4. **パフォーマンス**: ゼロコスト抽象化の実現
5. **後方互換性**: 既存APIを破壊しない

## 設計アプローチ

### 基本設計思想

本実装では **型状態パターン（Type State Pattern）** を採用し、以下の原則に基づいて設計する：

#### 1. コンパイル時安全性の最大化

Rustの型システムを活用し、実行時エラーを可能な限りコンパイル時エラーに変換する：

```rust
// コンパイル時に全パラメータの設定を強制
let query = GetUser::builder()
    .user_id(123)          // 必須パラメータ
    .include_deleted(false) // 必須パラメータ
    .build();              // 全パラメータ設定済みの場合のみコンパイル成功
```

#### 2. ゼロコスト抽象化の実現

builderパターンの実装において、実行時コストを排除する：

- `Option<T>`を使わずに型状態でフィールドの設定状態を管理
- `unwrap()`等の実行時チェックを排除
- 最終的な`build()`メソッドは単純なメモリ操作のみ

#### 3. copy_types最適化の統合

既存の`copy_types`設定と連携し、パラメータの渡し方を最適化する：

- Copy型（i32, bool等）: 値渡しで効率化
- 非Copy型（String等）: `Cow<'a, T>`で借用/所有を適応的に選択

## 実装戦略

### 1. 型状態パターンの実装

#### 基本構造

```rust
// クエリパラメータを保持する最終構造体
pub struct GetUser<'a> {
    user_id: i32,                    // Copy型：値渡し
    name: Cow<'a, str>,             // 非Copy型：Cowで最適化
}

// 型状態を管理するbuilder
pub struct GetUserBuilder<'a, Fields = ((), ())> {
    fields: Fields,                  // 実際の値を型レベルで管理
    phantom: PhantomData<&'a ()>,   // ライフタイム保持
}
```

#### 段階的型状態の変化

```rust
// 初期状態：未設定
GetUserBuilder<'a, ((), ())>

// user_id設定後：第1フィールドのみ設定済み
GetUserBuilder<'a, (i32, ())>

// name設定後：全フィールド設定済み
GetUserBuilder<'a, (i32, Cow<'a, str>)>
```

#### ゼロコストbuilder実装

```rust
impl<'a, V2> GetUserBuilder<'a, ((), V2)> {
    pub fn user_id(self, user_id: i32) -> GetUserBuilder<'a, (i32, V2)> {
        let ((), name) = self.fields;
        GetUserBuilder {
            fields: (user_id, name),  // 単純な値の移動のみ
            phantom: PhantomData,
        }
    }
}

// build()メソッド：unwrap()なしの単純なdestructuring
impl<'a> GetUserBuilder<'a, (i32, Cow<'a, str>)> {
    pub fn build(self) -> GetUser<'a> {
        let (user_id, name) = self.fields;  // ゼロコスト
        GetUser { user_id, name }
    }
}
```

### 2. QueryAnnotationに基づく実行メソッド

SQLCのクエリアノテーションに基づいて、適切な実行メソッドを生成する：

```rust
impl<'a> GetUser<'a> {
    // :one アノテーション → query_one & query_opt
    pub async fn query_one(&self, client: &Client) -> Result<UserRow, Error>
    pub async fn query_opt(&self, client: &Client) -> Result<Option<UserRow>, Error>
    
    // :many アノテーション → query_many & query_raw  
    pub async fn query_many(&self, client: &Client) -> Result<Vec<UserRow>, Error>
    pub async fn query_raw(&self, client: &Client) -> Result<impl Iterator<Item=Result<UserRow, Error>>, Error>
    
    // :exec アノテーション → execute
    pub async fn execute(&self, client: &Client) -> Result<u64, Error>
}
```

### 3. copy_types最適化の統合

既存の`copy_types`設定と統合し、型ごとに最適なパラメータ渡しを実現する：

#### 自動判定ルール

1. **Copy trait実装型かつ16バイト以下**: 値渡し
2. **String系**: `Cow<'a, str>`で最適化
3. **ユーザー定義copy_types**: 設定に従って値渡し
4. **その他**: 参照渡し

#### 実装例

```rust
// 生成される構造体（copy_types最適化適用後）
pub struct CreateUser<'a> {
    id: i32,                        // Copy型：値渡し
    name: Cow<'a, str>,            // 文字列：Cowで最適化
    email: Cow<'a, str>,           // 文字列：Cowで最適化
    is_active: bool,               // Copy型：値渡し
    metadata: &'a serde_json::Value, // 大きな型：参照渡し
}
```

## 後方互換性戦略

### 既存APIの保持

新しい構造体ベースAPIは**追加機能**として実装し、既存の関数ベースAPIを破壊しない：

```rust
// 既存API：そのまま維持（内部実装のみ最適化）
pub async fn get_user(client: &Client, user_id: i32, name: &str) -> Result<UserRow, Error> {
    // 内部でGetUser構造体を使用（外部には露出しない）
    let query = GetUser {
        user_id,
        name: Cow::Borrowed(name),
    };
    query.query_one(client).await
}

// 新しいAPI：オプションとして提供
let user = GetUser::builder()
    .user_id(123)
    .name("test")
    .build()
    .query_one(&client)
    .await?;
```

### 段階的移行戦略

1. **フェーズ1**: 構造体ベースAPIの実装と検証
2. **フェーズ2**: ドキュメント整備とサンプル提供
3. **フェーズ3**: ユーザーフィードバックに基づく改善
4. **フェーズ4**: 既存APIの非推奨化（長期計画）

## 実装の技術的詳細

### 1. コード生成フロー

```
SQL解析 → パラメータ抽出 → copy_types適用 → 構造体生成 → builder生成 → 実行メソッド生成
```

#### 主要な生成フェーズ

1. **パラメータ分析**: SQLクエリからパラメータを抽出
2. **型最適化**: copy_types設定に基づく型変換
3. **構造体生成**: 最適化された型でクエリ構造体を生成
4. **builder生成**: 型状態パターンでbuilderを生成
5. **メソッド生成**: QueryAnnotationに基づく実行メソッド生成

### 2. 実装上の制約と対処

#### 制約1: Rustの型システムの限界

**問題**: タプルの要素数には限界があるため、極端に多いパラメータを持つクエリでは型状態パターンが適用できない。

**対処**: パラメータ数が閾値（例：16個）を超える場合は、従来の`Option<T>`ベースのbuilderにフォールバック。

#### 制約2: ライフタイム管理の複雑さ

**問題**: 複数の借用パラメータが存在する場合、ライフタイム管理が複雑になる。

**対処**: 
- `PhantomData<&'a ()>`でライフタイムを統一
- 必要に応じて複数ライフタイムパラメータを使用
- `Cow<'a, T>`で借用と所有を適応的に切り替え

### 3. パフォーマンス考慮事項

#### ゼロコスト抽象化の検証

以下の観点でパフォーマンスを検証する：

1. **コンパイル時最適化**: リリースビルドでbuilderパターンが完全に最適化されるか
2. **メモリ使用量**: 構造体のメモリレイアウトが効率的か
3. **実行時コスト**: 従来の関数型APIと比較して性能劣化がないか

#### 最適化指針

```rust
// 最適化前（実行時コストあり）
struct Builder {
    user_id: Option<i32>,    // Option型によるオーバーヘッド
    name: Option<String>,    // unwrap()による実行時チェック
}

// 最適化後（ゼロコスト）
struct Builder<Fields = ((), ())> {
    fields: Fields,          // 型レベルでの状態管理
    phantom: PhantomData<()>, // コンパイル時のみ
}
```

## 設定オプションの拡張

### 既存設定の活用

構造体ベースAPIは既存の`copy_types`設定を活用し、新たな設定項目は追加しない方針とする。これにより設定の複雑化を避け、シンプルな利用体験を維持する。

```json
{
  "plugins": [
    {
      "name": "rust-postgres", 
      "options": {
        "copy_types": ["uuid::Uuid", "CustomId"],
        "existing_options": "..."
      }
    }
  ]
}
```

## 今後の拡張計画

### 短期計画（3ヶ月以内）

1. **基本実装の完成**: 型状態パターンの実装
2. **copy_types統合**: 既存最適化との連携
3. **テストケース整備**: 包括的なテストスイート
4. **ドキュメント整備**: API仕様とサンプルコード

### 中期計画（6ヶ月以内）

1. **エラーメッセージ改善**: コンパイルエラーの分かりやすさ向上
2. **IDE統合**: rust-analyzerでの補完とエラー表示改善
3. **パフォーマンス検証**: ベンチマークとプロファイリング
4. **エコシステム対応**: 主要なPostgreSQLクレートとの互換性確認

### 長期計画（1年以内）

1. **バッチ操作対応**: BatchExec等のアノテーション対応
2. **トランザクション統合**: 型安全なトランザクション管理
3. **マイグレーション支援**: 既存コードの移行ツール
4. **他データベース対応**: MySQL、SQLite等への展開

## リスク分析と対策

### 技術的リスク

#### リスク1: コンパイル時間の増加

**原因**: 複雑な型状態パターンによるコンパイル負荷増加

**対策**: 
- 型の複雑さを適切に制限
- インクリメンタルコンパイルの最適化
- 必要に応じてマクロによる実装簡素化

#### リスク2: エラーメッセージの複雑化

**原因**: 型状態パターンの型エラーが分かりにくい

**対策**:
- 適切な型エイリアスの提供
- カスタムコンパイルエラーメッセージ
- 詳細なドキュメントと例示

### 採用リスク

#### リスク1: 学習コストの増加

**原因**: 新しいAPIパターンの習得が必要

**対策**:
- 段階的な移行計画
- 豊富なサンプルコードとチュートリアル
- 既存APIとの併用期間の確保

#### リスク2: エコシステムの分断

**原因**: 新旧APIの並存による混乱

**対策**:
- 明確な移行ガイドライン
- ツールによる自動変換支援
- コミュニティとの十分な対話

## 結論

構造体ベースAPIの導入により、以下の価値を提供できる：

1. **開発者体験の向上**: 型安全性と可読性の大幅な改善
2. **保守性の向上**: SQLクエリ変更に対する堅牢性
3. **パフォーマンス**: ゼロコスト抽象化による効率性
4. **段階的移行**: 既存コードへの影響を最小化

本実装は、Rustの型システムの力を最大限に活用し、データベースアクセスレイヤーの安全性と効率性を大幅に向上させる重要な機能である。慎重な実装と十分な検証を通じて、sqlc-rust-postgresプラグインの価値を更に高めることができると確信している。

---

*本文書の内容は実装の進行に伴い更新される予定である。最新の情報については、GitHubのissueとPRを参照されたい。*