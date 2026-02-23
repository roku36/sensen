---
name: add-card
description: 新しいカードをSensenに追加する。CardId、CardDef登録、CardEffect、mesa表示を一括で設定する。
user-invocable: true
allowed-tools: Read, Edit, Grep, Glob
argument-hint: [card-name] [type:attack|skill|power|status]
---

# Sensen Add Card

新しいカードを追加する手順。すべてのファイルを正しく更新する。

## 必要な変更箇所

### 1. CardId追加 (`src/game/cards/mod.rs`)
CardId enumに新しいバリアントを追加:
- Attack: 1-99
- Skill: 100-199
- Power: 200-299
- Status: 300-399

既存の最大IDを確認して次の番号を使う。

### 2. CardDef登録（タイプに応じたファイル）
- Attack → `src/game/cards/attack.rs` の `register_attack_cards()`
- Skill → `src/game/cards/skill.rs` の `register_skill_cards()`
- Power → `src/game/cards/power.rs` の `register_power_cards()`
- Status → `src/game/cards/status.rs` の `register_status_cards()`

```rust
registry.register(CardDef {
    id: CardId::NewCard,
    name: "New Card".to_string(),
    description: "Does something cool".to_string(),
    card_type: CardType::Attack,
    rarity: CardRarity::Common,
    cost: 1.0,
    effect: CardEffect::Damage(50.0),
});
```

### 3. CardEffect（新しいエフェクトの場合）
新しいエフェクトが必要なら:
1. `src/game/cards/mod.rs` の `CardEffect` enumにバリアント追加
2. `src/game/effect.rs` の `apply_card_effect()` にmatchアーム追加
3. `src/game/mesa.rs` の以下を更新:
   - `mesa_card_from_id()` → カード画像マッピング
   - `unified_effect_kind()` → エフェクト分類
   - `effect_lines()` → カード上テキスト
   - `effect_color()` → テキスト色

### 4. テストデッキに追加（任意）
`src/game/player.rs` の `create_test_deck()` に追加してテスト可能にする。

### 5. Power効果コンポーネント（Powerカードの場合）
新しい永続効果が必要なら:
1. `src/game/status.rs` にコンポーネント定義
2. `src/network/mod.rs` に `rollback_component_with_clone` 追加
3. `src/game/status.rs` の tick システムに処理追加
4. `src/game/ui.rs` の `build_status_string()` に表示追加

## Combo効果
複数の効果を組み合わせるには `CardEffect::Combo(vec![...])` を使う:
```rust
effect: CardEffect::Combo(vec![
    CardEffect::Damage(80.0),
    CardEffect::Block(50.0),
]),
```

## 注意点
- Strengthはダメージに `strength * 10.0` で加算される
- Weakはダメージを0.75倍、Vulnerableは被ダメージを1.5倍
- Powerカードはプレイするとデッキに戻らない
- Exhaustはデッキからも消える（戦闘中のみ）
- CorruptionでSkillは0コスト+Exhaust化
