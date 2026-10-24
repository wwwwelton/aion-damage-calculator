---
title: Melhoria de Atributos, Telas e Fórmula de Dano - Plan
type: feat
date: 2026-10-24
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-plan-bootstrap
execution: code
---

# Melhoria de Atributos, Telas e Fórmula de Dano - Plan

## Goal Capsule

- **Objective:** Atualizar o aplicativo para exibir com precisão os atributos consolidados dos perfis de AION 2 (com Avatarnoia como referência), distinguindo bônus publicados de parciais e aprimorando a transparência da calculadora e da interface.
- **Means:** Refatorar os módulos de importação (`characters.js`), renderização de perfil (`profile-ui.js`), dicas (`ai-tips.js`) e motor de cálculo (`calculator.js`), unificando fontes oficiais (Metabot, Shugo, Aionflex e PlayNC) e marcando regras não comprovadas como hipóteses estimadas.
- **Authority:** Solicitação explícita do usuário para incorporar os dados dos quatro perfis, melhorar as telas e refinar a fórmula de dano sem sincronização automática nova.
- **Stop condition:** Não inventar coeficientes de dano não documentados no cliente e não afirmar que a fórmula cobre taxas de ativação complexas de forma exata.
- **Execution:** `ce-work` implementa e verifica localmente; a publicação final exige validação no ambiente de testes.

---

## Product Contract

### Summary

Melhorar a experiência de consulta e simulação no aplicativo de AION 2, permitindo importar e detalhar corretamente todos os atributos de personagem (como os de Avatarnoia) e explicitando quais termos afetam o dano por golpe versus o DPS estimado.

### Problem Frame

O aplicativo atual possui lacunas na importação de certos atributos secundários dos perfis públicos (como bônus de divindade de Destruction e bônus de precisão de Precision) e mistura bônus publicados com estimativas hipotéticas na calculadora, gerando confusão sobre a exatidão do dano simulado.

### Requirements

- R1. O importador de perfil (`characters.js`) mapeia corretamente todos os atributos de fichas públicas e bônus de divindade (incluindo Destruction e Precision), evitando duplicações e separando valores publicados de parciais.
- R2. A interface de perfil (`profile-ui.js`) e as árvores de atributos (`attribute-trees.js`) distinguem claramente valores fixos, percentuais e bônus derivados.
- R3. A calculadora de dano (`calculator.js`) e os testes (`calculator.test.js`) diferenciam o dano por golpe de uma estimativa probabilística de combate, sinalizando claramente hipóteses e restrições estatísticas.

### Key Decisions

- **Referência em Avatarnoia** (session-settled: user-directed — chosen over perfil genérico: usar Avatarnoia como validação prática para Clérigo nível 45 em Siel). Governs R1, R2, R3.

### Scope Boundaries

- **In scope:** Correção de mapeamentos de atributos, refinamento visual das telas de ficha, clareza na separação entre dano de golpe e estimativa de DPS, testes de regressão.
- **Deferred to Follow-Up Work:** Sincronização automática em segundo plano com as quatro plataformas externas (mantém consulta pontual sob demanda).

---

## Planning Contract

### Key Technical Decisions

- **KTD1. Mapeamento robusto de bônus de divindade e atributos principais:** Estender o dicionário em `characters.js` para cobrir todos os atributos de divindade e primários identificados nas fontes oficiais, garantindo que bônus de Ataque (Might e Destruction) somem corretamente sem conflito com o ataque base.
- **KTD2. Distinção explícita de estimativas na UI:** Atualizar `renderer.js` e `i18n.js` para rotular claramente o que é dano determinístico de um golpe e o que é simulação hipotética com taxas de ativação.

### Implementation Units

### U1. Expandir Mapeamento de Atributos e Correções no Importador
- **Goal:** Garantir que o importador reconheça todos os atributos secundários e de divindade presentes nas fichas públicas (Metabot, Shugo, Aionflex e PlayNC).
- **Requirements:** R1
- **Dependencies:** None
- **Files:** `characters.js`, `characters.test.js`
- **Approach:** Atualizar os dicionários de mapeamento em `characters.js` para incluir todas as chaves de divindade (como Destruction para ataque e Precision para acerto/crítico) e validar com testes em `characters.test.js`.
- **Test scenarios:**
  - Importar perfil com bônus de Destruction e Precision e verificar se refletem nos campos corretos.
  - Validar que atributos desconhecidos são ignorados sem quebrar o parser.
- **Verification:** `node --test characters.test.js` passa sem erros.

### U2. Aprimorar Telas de Ficha e Árvores de Atributos
- **Goal:** Organizar a exibição de atributos no perfil para distinguir o que vem da ficha oficial, o que é de equipamento e o que é bônus derivado.
- **Requirements:** R2
- **Dependencies:** U1
- **Files:** `profile-ui.js`, `attribute-trees.js`
- **Approach:** Ajustar a renderização dos cartões de atributos em `profile-ui.js` para exibir claramente a procedência e os efeitos secundários publicados, mantendo a consistência com `attribute-trees.js`.
- **Test scenarios:**
  - Renderizar perfil com atributos mistos e verificar se os rótulos refletem a origem correta.
- **Verification:** Verificação visual via smoke teste do app e testes unitários existentes.

### U3. Refinar Transparência da Fórmula e Estimativas na Calculadora
- **Goal:** Tornar explícitas na interface da calculadora e nas dicas as distinções entre dano por golpe, taxas de proc e hipóteses estatísticas.
- **Requirements:** R3
- **Dependencies:** U1, U2
- **Files:** `calculator.js`, `renderer.js`, `i18n.js`, `calculator.test.js`
- **Approach:** Atualizar os textos de status e notas explicativas em `i18n.js` e `renderer.js` para deixar claro o escopo da simulação de DPS e o tratamento de resistências e procs.
- **Test scenarios:**
  - Verificar cálculo de dano com diferentes modos de alvo (Boss, Monstro, Jogador) e garantir que as notas explicativas aparecem corretamente.
- **Verification:** `node --test *.test.js` e `node ai-model.smoke.mjs` passam com sucesso.

---

## Verification Contract

- Executar a bateria de testes unitários: `node --test *.test.js`
- Executar o teste de fumaça de inferência: `node ai-model.smoke.mjs`
- Conferir o build de IA e a publicação: `npm run build:ai` e `dotnet publish Aion2Calculator.csproj -c Release -o dist/webview2`

## Definition of Done

- Todos os itens do plano implementados e verificados localmente.
- Nenhuma modificação em arquivos de log ou resíduos experimentais no diff.
