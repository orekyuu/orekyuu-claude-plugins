# テスト方針（Java）

共通方針は [test-policy.md](test-policy.md) を参照してください。

---

## テストフレームワーク・ツール

| 用途                         | ツール         |
| ---------------------------- | -------------- |
| テストフレームワーク         | JUnit 5        |
| アサーション                 | AssertJ        |
| モック                       | Mockito        |
| コンテナ起動（RDB・MQ など） | Testcontainers |
| 外部 API の Fake             | WireMock       |

## テストクラス・メソッドの命名規則

- テストクラス名は `XxxTest`（テスト対象クラス名 + `Test`）
- `@DisplayName` で日本語の説明を付ける
- テスト名は共通方針の「XXXのとき、YYYになること」に従う

```java
@DisplayName("OrderService")
class OrderServiceTest {

    @Test
    @DisplayName("在庫が不足しているとき、OutOfStock エラーになること")
    void outOfStock() { ... }
}
```

## アサーションの書き方

- `assertThat`（AssertJ）を使う。JUnit の `assertEquals` / `assertTrue` は使わない
- Result / Either 型の検証はまず成功・失敗を確認してから値を検証する

```java
// 成功ケース
assertThat(result.isSuccess()).isTrue();
assertThat(result.getValue()).isEqualTo(expected);

// 失敗ケース
assertThat(result.isFailure()).isTrue();
assertThat(result.getError()).isInstanceOf(InsufficientBalanceError.class);

// 例外が投げられることを確認（引数不正のテスト）
assertThatThrownBy(() -> new Email(null))
    .isInstanceOf(IllegalArgumentException.class);
```

## モックの使い方

- Domain 層のテストではインターフェースのインメモリ実装を用意して使う。Mockito は使わない
- Application 層のテストでは `@Mock` + `@InjectMocks` で Domain 層のインターフェースをモックする
- Spring のコンテキストを起動する `@MockBean` は統合テスト限定とし、単体テストでは使わない

```java
// Domain 層のテスト: インメモリ実装を使う
var repository = new InMemoryUserRepository();
var service = new UserRegistration(repository);

// Application 層のテスト: Mockito でインターフェースをモック
@Mock UserRepository userRepository;
@InjectMocks UserApplicationService sut;
```

## Testcontainers の使い方

- RDB・Redis・MQ などの実装が絡む Infrastructure 層のテストで使用する
- `@Testcontainers` + `@Container` でクラス単位またはスイート単位でコンテナを共有する
- Domain 層・Application 層のテストには使わない（インメモリ実装やモックで代替する）

## WireMock の使い方

- 外部 API との通信を含む Gateway 実装のテストで使用する
- リクエストのマッチング条件と期待レスポンスをスタブとして定義する
- Domain 層・Application 層のテストには使わない
