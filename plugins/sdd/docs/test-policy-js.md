# テスト方針（JavaScript / TypeScript）

共通方針は [test-policy.md](test-policy.md) を参照してください。

---

## テストフレームワーク・ツール

| 用途                 | ツール                     |
| -------------------- | -------------------------- |
| テストフレームワーク | Vitest（推奨）または Jest  |
| コンポーネントテスト | Testing Library            |
| ネットワークモック   | MSW（Mock Service Worker） |
| E2E テスト           | Playwright                 |

### Vitest と Jest の選択基準

- **Vitest を選ぶ**: Vite ベースのプロジェクト（Vite / Nuxt / SvelteKit など）。設定が少なく ESM ネイティブ対応
- **Jest を選ぶ**: Vite を使わない Node.js プロジェクト、または既存プロジェクトがすでに Jest を使っている場合

## テストファイルの配置

テスト対象ファイルと同階層に `.test.ts` を置く。

```
src/
└── users/
    ├── UserRegistration.ts
    ├── UserRegistration.test.ts   ← 同階層に置く
    └── InMemoryUserRepository.ts
```

`__tests__/` ディレクトリへのまとめ置きはしない。対象ファイルとテストが離れると追跡しにくくなるため。

## アサーションの書き方

- `expect` を使う。Result / Either 型は成功・失敗を確認してから値を検証する

```ts
// 成功ケース
const result = register(input);
expect(result.ok).toBe(true);
expect(result.value).toEqual(expected);

// 失敗ケース
const result = register(invalidInput);
expect(result.ok).toBe(false);
expect(result.error).toBeInstanceOf(DuplicateUserError);

// 引数不正（Parse 境界でのテスト）
expect(() => Email.parse("")).toThrow(InvalidEmailError);
```

## モックの使い方

- Domain 層のテストではインターフェースのインメモリ実装を用意して使う。`vi.mock` / `jest.mock` は使わない
- `vi.mock` / `jest.mock` はモジュール単位のモックが必要な場面（外部ライブラリの差し替えなど）に限定する
- 外部 API との通信は MSW でネットワーク層をインターセプトしてモックする

```ts
// Domain 層のテスト: インメモリ実装を使う
const repository = new InMemoryUserRepository();
const useCase = new RegisterUser(repository);

// 外部 API のテスト: MSW でレスポンスをスタブ
server.use(
  http.post("/api/payment", () => HttpResponse.json({ status: "ok" })),
);
```

## コンポーネントテストの方針

- Testing Library を使い、DOM の内部実装ではなくユーザーの視点（テキスト・ロール・ラベル）でクエリする
- `getByRole` / `getByLabelText` / `getByText` を優先し、`getByTestId` は最終手段とする
- ユーザーイベントは `userEvent`（`@testing-library/user-event`）を使い、`fireEvent` は使わない

```ts
test("メールアドレスが空のとき、エラーメッセージが表示されること", async () => {
  render(<RegisterForm />);
  await userEvent.click(screen.getByRole("button", { name: "登録" }));
  expect(screen.getByText("メールアドレスを入力してください")).toBeInTheDocument();
});
```
