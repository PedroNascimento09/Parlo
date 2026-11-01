# PARTE XIV · INTERFACE, IDENTIDADE VISUAL E EXPERIÊNCIA

Esta parte fixa o que até aqui estava disperso: a aparência do Parlo, os ecrãs que existem, a forma de navegar entre eles e o que cada um mostra em cada estado. As secções 2, 10, 11, 12, 13, 14, 37, 40, 44 a 55, 63, 65 a 67 e 71 continuam a valer; aqui são especificadas ao nível do pixel e do fluxo. Nenhum requisito técnico, de privacidade ou de voz é alterado por esta parte, e em caso de conflito prevalecem as Partes II, V, IX e XII.

A regra que organiza tudo o resto: **a janela principal é o centro de configuração e de leitura; o trabalho acontece em superfícies pequenas junto ao contexto**. Quem dita não abre a janela; quem configura modelos abre-a e encontra tudo arrumado.

## 83. Personalidade visual e direção artística

### 83.1. Posicionamento

O Parlo é uma ferramenta de trabalho, não uma demonstração de modelos. A interface tem de transmitir três coisas e nenhuma quarta: **voz humana** (algo caloroso, não clínico), **fiabilidade local** (nada de nuvem, nada de conta, nada a piscar sem motivo) e **contenção** (o produto desaparece quando está a funcionar).

| Atributo | O que significa na prática | O que nunca significa |
|---|---|---|
| Caloroso | Papel quente em vez de branco frio, acento terracota, tipografia com ritmo de leitura | Ícones de microfone cor-de-rosa, ilustrações infantis |
| Sóbrio | Uma cor de acento por ecrã, hierarquia por peso e tamanho | Cinzento sem contraste, ecrãs todos iguais |
| Rápido | Estados que mudam em menos de 200 ms, texto que aparece a crescer | Animações decorativas, esqueletos que dançam |
| Honesto | O estado real escrito por palavras, incluindo o que falhou | Gráficos que sugerem precisão que não existe |
| Local | A frase «fica neste computador» visível onde importa | Selos de privacidade em todos os cantos |

### 83.2. Direção artística: papel e voz

A metáfora é **papel e voz**. O fundo é papel quente, o texto é tinta, o acento é a cor de uma gravação a acontecer. Superfícies são planas com uma sombra curta, as regras são linhas de 1 px, e o que se move é o texto, não a caixa.

Consequências concretas:

- Sem gradientes em superfícies de trabalho. Um gradiente suave é permitido apenas no ecrã de arranque e no onboarding.
- Sem vidro fosco excepto na ilha de ditado e nas legendas, onde a transparência tem função: o conteúdo por baixo continua legível.
- Sem ícones a cores. Um só traço, 1,5 px, cantos redondos, 20 × 20 na escala base.
- Sem cartões dentro de cartões. Uma superfície, uma sombra, um assunto.
- Sem métricas no ecrã inicial. O que não ajuda a próxima ação não aparece.

### 83.3. Densidade e ritmo

Três densidades, aplicadas por superfície e nunca por preferência global:

| Densidade | Onde | Altura de linha de lista | Espaço entre secções |
|---|---|---|---|
| Confortável | Home, Voice, Definições | 44 px | 32 px |
| Compacta | Model Hub, Histórico, Meeting | 32 px | 20 px |
| Mínima | Ilha de ditado, legendas | 24 px | 8 px |

## 84. Marca: logótipo, símbolo e uso

O símbolo é um **ponto de gravação dentro de um arco aberto**: um círculo de 10 px de diâmetro e um arco de 240° com 1,75 px de espessura, ambos em acento, dentro de um quadrado de 24 px com raio de 7 px. Lê-se como som a sair de uma boca sem desenhar uma boca, e sobrevive a 16 px na barra de tarefas.

| Contexto | Forma | Regra |
|---|---|---|
| Ícone da aplicação | Símbolo sobre papel quente, canto arredondado | Sem texto, sem sombra |
| Barra superior da janela | Símbolo 20 px + palavra «Parlo» a 15 px, peso 620 | A palavra nunca é substituída pelo símbolo sozinho |
| Ilha de ditado | Ponto de estado, sem símbolo | O símbolo ocuparia espaço de leitura |
| Documentação e releases | Símbolo + palavra, alinhados à esquerda | Nunca centrados numa página de texto corrido |
| Estado de erro | Símbolo em tinta, sem acento | O acento comunica gravação, não falha |

Regras de marca: o acento nunca é usado como cor de texto corrido; o símbolo nunca roda nem pulsa; a palavra «Parlo» escreve-se sempre com maiúscula inicial e sem aspas; não existe versão do logótipo a branco sobre o acento.

## 85. Design tokens

Os tokens são a única fonte de verdade visual. Um valor fora desta tabela é um erro de implementação, não uma escolha.

### 85.1. Cor, tema claro

| Token | Valor | Uso |
|---|---|---|
| `canvas` | `#FAF6EF` | Fundo da janela |
| `surface` | `#FFFFFF` | Cartões, painéis, campos |
| `surface-sunken` | `#F1EBE1` | Zonas inertes, cabeçalhos de tabela |
| `ink` | `#1B1613` | Texto principal |
| `ink-soft` | `#3E362F` | Texto secundário com contraste AA |
| `muted` | `#6B6257` | Legendas, metadados, notas |
| `line` | `#E5DCCD` | Bordas de 1 px e separadores |
| `accent` | `#D9481F` | Ação primária, gravação, foco |
| `accent-deep` | `#A8330F` | Hover e active da ação primária |
| `accent-wash` | `#FBEDE7` | Fundo de destaque e seleção |
| `listen` | `#1F6F6B` | Escuta activa, forma de onda |
| `ok` | `#2F7A4F` | Concluído, guardado, verificado |
| `warn` | `#B4761A` | Aviso, degradação, limite |
| `error` | `#A32B22` | Falha, bloqueio, ação destrutiva |
| `tech` | `#1F3A4D` | Detalhe técnico, identificadores, mono |

### 85.2. Cor, tema escuro

| Token | Valor | Nota |
|---|---|---|
| `canvas` | `#141110` | Papel à noite, não preto puro |
| `surface` | `#1D1917` | Uma elevação acima do fundo |
| `surface-sunken` | `#221D1A` | Zonas inertes |
| `ink` | `#F5EFE6` | Contraste 13,4:1 sobre `canvas` |
| `ink-soft` | `#D8CEC0` | Contraste 9,1:1 |
| `muted` | `#A79C8E` | Contraste 5,6:1, mínimo AA |
| `line` | `#302925` | Bordas |
| `accent` | `#F2693A` | Mais luminoso porque o fundo é escuro |
| `accent-deep` | `#FF8154` | Hover |
| `accent-wash` | `#2A1C16` | Seleção |
| `listen` | `#56B8B1` | Escuta |
| `ok` | `#5FBF86` | Concluído |
| `warn` | `#E0A548` | Aviso |
| `error` | `#E8665B` | Falha |
| `tech` | `#9CB3C4` | Detalhe técnico |

Regra de contraste: nenhum par texto/fundo fica abaixo de 4,5:1 para texto normal e 3:1 para texto grande ou elementos de interface. `muted` sobre `surface` é o par mais frágil e por isso é verificado em cada alteração de tema.

### 85.3. Tipografia

| Token | Tamanho / linha | Peso | Uso |
|---|---|---|---|
| `display` | 32 / 38 | 600 | Título do onboarding e de ecrãs vazios |
| `title` | 24 / 30 | 620 | Título de secção da janela |
| `heading` | 18 / 24 | 600 | Título de cartão e de painel |
| `body` | 14 / 21 | 400 | Texto corrido e listas |
| `body-strong` | 14 / 21 | 600 | Ênfase e rótulos de valor |
| `small` | 12,5 / 18 | 400 | Notas, metadados, ajuda |
| `caption` | 11,5 / 16 | 500 | Cabeçalhos de tabela, estados |
| `mono` | 12 / 16 | 400 | Tempos, hashes, latências |

Família: `"Segoe UI", system-ui, -apple-system, sans-serif` na interface; `"Cascadia Mono", ui-monospace, monospace` em tudo o que é número técnico. Números com `font-variant-numeric: tabular-nums` para não dançar durante a gravação. Texto corrido nunca justificado na interface (só no documento do plano).

### 85.4. Espaço, raio, elevação, movimento

| Escala | Valores | Regra |
|---|---|---|
| Espaço | 4, 8, 12, 16, 24, 32, 48, 64 px | Margem interior de cartão: 20 px; entre cartões: 16 px |
| Raio | 8, 12, 18, 26, 999 px | Campo e botão: 12; cartão: 18; ilha: 26; pílula: 999 |
| Elevação | 0, 1, 2 | 0 plano; 1 `0 1px 0 rgba(27,22,19,.04), 0 6px 18px rgba(27,22,19,.06)`; 2 `0 18px 40px rgba(0,0,0,.18)` só em sobreposições |
| Movimento | 120 / 180 / 260 ms | 120 para hover e foco; 180 para entrada de conteúdo; 260 para painéis |
| Curva | `cubic-bezier(.2,.9,.3,1)` | A mesma em todo o produto |

Movimento permitido: aparecer, desaparecer, expandir em altura, barra de progresso, forma de onda. Movimento proibido: parallax, rotação contínua, pulsação de mais de um elemento por ecrã, qualquer animação que dure mais de 300 ms. Com `prefers-reduced-motion`, todas as transições passam a 0 ms excepto a barra de progresso, que continua porque comunica estado.

## 86. Componentes reutilizáveis

Um só conjunto, usado por todos os ecrãs. Cada componente tem estados de repouso, hover, foco, active, disabled, loading e erro, e nenhum ecrã inventa uma variante local.

| Componente | Anatomia | Regras |
|---|---|---|
| Botão primário | Fundo `accent`, texto `#FFFFFF`, raio 12, altura 36 | Um por ecrã; `disabled` com 45% de opacidade e cursor normal |
| Botão secundário | Fundo `surface`, borda `line`, texto `ink` | Para tudo o que não é a ação principal |
| Botão destrutivo | Texto `error`, fundo transparente, borda `line` | Só confirma depois de um diálogo com o nome do que é apagado |
| Botão de ícone | 32 × 32, raio 10, ícone 18 | Obrigatório `aria-label` |
| Cartão | Superfície, raio 18, elevação 1, 20 px de margem | Sem cartões aninhados |
| Badge de estado | Pílula 999, altura 20, texto `caption` | Sempre com palavra, nunca só cor |
| Campo de texto | Altura 36, borda 1 px, raio 12, foco com anel de 2 px `accent` | Erro com borda `error` e mensagem abaixo |
| Seletor | Igual ao campo, com seta de 14 px | Abre em lista com 8 px de folga, sombra 2 |
| Separador | Linha `line` de 1 px | Nunca uma caixa à volta de texto |
| Barra de progresso | 6 px de altura, raio 999, `accent` sobre `surface-sunken` | Percentagem em `mono` quando o total é conhecido |
| Medidor de áudio | 24 barras de 3 px, altura 4 a 20 px, cor `listen` | Reage à amplitude RMS; não é decorativo |
| Indicador de estado | Ponto de 8 px + palavra | Ponto sozinho só na ilha, onde a palavra está ao lado |
| Diálogo | 480 px de largura, raio 18, sombra 2 | Duas ações no máximo, a destrutiva à direita |
| Notificação | Faixa no topo da área de trabalho, 44 px, some em 6 s | Erros não desaparecem sozinhos |
| Tooltip | 240 px, `small`, 120 ms de atraso | Nunca contém ação |
| Lista | Linhas de 32 a 44 px, separador `line` | Ordenação indicada no cabeçalho |
| Tabela | Cabeçalho `surface-sunken`, `caption` | Máximo 5 colunas na largura mínima |
| Painel lateral | 320 px, `surface`, borda esquerda `line` | Fecha com Esc e devolve o foco ao gatilho |

## 87. Estados globais

Estes dez estados têm a mesma apresentação em qualquer superfície. Não é permitido um ecrã inventar uma cor ou uma palavra diferente para o mesmo estado.

| Estado | Ponto | Palavra | Cor | O que a interface faz |
|---|---|---|---|---|
| `idle` | Cheio `muted` | Ready | `muted` | Nada se move |
| `listening` | Pulsante `accent` | Listening | `accent` | Medidor activo, contador de tempo |
| `processing` | Meio `tech` | Writing | `tech` | Barra indeterminada, texto anterior esbatido |
| `translating` | Meio `tech` | Translating | `tech` | Mostra o original já reconhecido |
| `speaking` | Pulsante `listen` | Speaking | `listen` | Forma de onda, botão de parar |
| `paused` | Vazio `warn` | Paused | `warn` | Diz porque pausou |
| `offline` | Vazio `muted` | Offline | `muted` | Explica que o local continua a funcionar |
| `warning` | Cheio `warn` | Attention | `warn` | Uma frase com a consequência |
| `error` | Cheio `error` | Failed | `error` | Frase com o que fazer a seguir |
| `completed` | Cheio `ok` | Done | `ok` | Marca desaparece em 4 s e o resultado fica |

Regra de acessibilidade do estado: a palavra está sempre presente, o `aria-live` é `polite` excepto em erro, onde é `assertive`, e o movimento do ponto nunca é o único sinal.

## 88. Arquitetura de navegação

### 88.1. Estrutura

A janela principal tem barra lateral fixa de 232 px, área de trabalho e, quando necessário, um painel de detalhe de 320 px. Não há menus em cascata nem separadores dentro de separadores.

| Grupo | Itens | Porque estão juntos |
|---|---|---|
| Trabalho | Home, Dictate, Captions, Live | O que o utilizador faz todos os dias |
| Voz | Voice | Perfis, consentimento, compatibilidade |
| Reuniões | Meeting | Lite, Live e Chat são o mesmo assunto |
| Biblioteca | Models, History | O que está instalado e o que já aconteceu |
| Sistema | Settings, Licences | Configuração e transparência |

A barra lateral colapsa para 64 px com ícones e tooltips; o estado é lembrado. O item activo tem fundo `accent-wash`, texto `ink` e uma barra de 3 px a `accent` na margem esquerda. Nunca há dois itens activos.

### 88.2. Barra superior

48 px de altura, com o nome do ecrã à esquerda em `heading`, e à direita três controlos permanentes: estado do motor (ponto + palavra), idiomas activos (`PT → EN` em `caption`), e o botão de atualização quando existe versão nova. O menu do utilizador não existe, porque não há contas; no lugar dele está um botão de ajuda que abre as definições no grupo de diagnóstico.

### 88.3. Regras de navegação

- Cada ecrã tem um endereço interno navegável por teclado e um atalho `Alt+1` a `Alt+5` para os cinco grupos.
- O botão Voltar do rato e `Alt+Esquerda` percorrem o histórico dentro da aplicação.
- Nenhuma ação destrutiva acontece por navegação: sair de um ecrã nunca apaga uma sessão.
- Uma tarefa em curso (ditado, legendas, live) mostra um indicador na barra lateral e continua se o utilizador mudar de ecrã.
- A janela recorda tamanho e posição, e a última secção aberta.
- Fechar a janela não interrompe uma sessão Live: a aplicação continua na bandeja e diz isso antes de fechar.

## 89. Parlo Home

O Home responde a três perguntas e mais nenhuma: *posso falar agora?*, *o que aconteceu na última vez?* e *há alguma coisa que eu tenha de fazer?*

```
┌──────────────────────────────────────────────────────────────────────┐
│ Parlo        ● Engine ready      PT → EN            [ Update ]       │
├────────────┬─────────────────────────────────────────────────────────┤
│ Trabalho   │  Good afternoon.                                        │
│  Home    ● │  Hold the shortcut and speak; the other language comes  │
│  Dictate   │  back in your voice.                                   │
│  Captions  │                                                         │
│  Live      │  ┌───────────────────────────────────────────────────┐  │
│ Voz        │  │  ◉  Hold to speak             Ctrl+Alt+D          │  │
│  Voice     │  │     Release to hear the translation   0.0 s        │  │
│ Reuniões   │  └───────────────────────────────────────────────────┘  │
│  Meeting   │                                                         │
│ Biblioteca │  Languages   PT → EN            [ swap ]               │
│  Models    │  Microphone  Realtek Array · -24 dBFS   [ test ]       │
│  History   │  Voice       My voice · ready                           │
│ Sistema    │                                                         │
│  Settings  │  Recent                                                 │
│  Licences  │  ┌───────────────────────────────────────────────────┐  │
│            │  │ 14:32  PT → EN   0.9 s   "A reunião é quinta…"    │  │
│            │  │ 14:18  Dictate   1.2 s   42 words copied          │  │
│            │  │ 13:55  Live      8 min   2 windows failed         │  │
│            │  └───────────────────────────────────────────────────┘  │
│            │  Models 4 installed · 2.3 GB · 18 GB free   [ manage ] │
└────────────┴─────────────────────────────────────────────────────────┘
```

Regras do ecrã:

- O cartão de gravação é o único elemento com `accent` e ocupa a primeira posição visual.
- O estado do motor aparece por palavras, e «Engine ready» só aparece quando o motor respondeu a um pedido real.
- Avisos de configuração aparecem acima do cartão de gravação, no máximo um de cada vez, com uma ação e um «Not now».
- «Recent» mostra no máximo cinco entradas e liga ao History.
- A linha de modelos mostra espaço e um botão «manage»; nunca uma lista de modelos no Home.
- Sem gráficos, sem contadores de utilização, sem percentagens de qualidade.

## 90. Parlo Dictate

### 90.1. Ilha flutuante

A ilha é uma janela própria, sem moldura, sempre acima das outras, com 560 × 96 px lógicos, centrada a 64 px do fundo do monitor onde está o cursor. Não rouba o foco: aparecer não activa a janela de destino nem a desactiva.

```
      ╭──────────────────────────────────────────────────────╮
      │  ●  LISTENING                      Speech → English  │
      │     Speak now, then press the shortcut again.  4.2 s │
      ╰──────────────────────────────────────────────────────╯
```

Estados da ilha, e só estes:

| Estado | Linha de estado | Linha de conteúdo | Acção da tecla |
|---|---|---|---|
| `idle` | READY | «Press Ctrl+Alt+D and speak.» | mostra e começa |
| `listening` | LISTENING | texto parcial a crescer | pára e escreve |
| `processing` | WRITING | «Your words are being written down.» | ignora |
| `done` | COPIED | texto final, uma linha, truncado com reticências | nova gravação |
| `empty` | NOTHING HEARD | «Speak a little closer to the microphone.» | nova gravação |
| `error` | FAILED | a razão concreta e uma saída | nova gravação |

Regras: sem botões dentro da ilha; a interação é o atalho e `Esc`. O texto copiado vive no clipboard e a ilha di-lo por palavras. A posição é memorizada por monitor; arrastar com o rato move a ilha e a posição fica guardada. Em ecrã inteiro de outra aplicação, a ilha continua visível por cima, porque é `alwaysOnTop`, e nunca entra em ecrã inteiro exclusivo.

### 90.2. Ecrã Dictate na janela

Serve para ditar frases longas, rever antes de inserir e escolher destino.

```
┌──────────────────────────────────────────────────────────────────────┐
│ Dictate                                                              │
│                                                                      │
│  I speak   [ Portuguese (Portugal) ▾ ]   Write  [ English ▾ ]        │
│  Style     ( Verbatim ) ( Clean ) ( Translated )        [ Formal ]   │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ BOA TARDE, VENHO CONFIRMAR A REUNIÃO DE QUINTA-FEIRA…          │  │
│  └────────────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ Good afternoon, I am writing to confirm Thursday's meeting…    │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  Numbers ✓   Names ✓   Negation ✓     PT→EN candidate, not validated │
│                                                                      │
│  [ Insert ]   [ Copy ]   [ Discard ]                 1.8 s          │
└──────────────────────────────────────────────────────────────────────┘
```

Regras: o verificador de números, nomes e negações aparece como três marcas com palavra; uma marca falhada fica a `warn` e explica qual valor mudou. Quando a tradução vier vazia, o botão primário passa a «Insert original» e diz porquê. Rever antes de inserir é a predefinição nos pares não validados; inserir automaticamente é uma opção por par, nunca global.

## 91. Parlo Captions

### 91.1. Barra de legendas

Janela própria, sem moldura, arrastável, com largura entre 480 e 1 200 px e altura conforme as linhas. Fundo `#101010` a 72% de opacidade, texto branco, sombra de texto fina para legibilidade sobre vídeo.

```
  ╭──────────────────────────────────────────────────────────────╮
  │  PT → EN · mic + system audio                    ● live      │
  │  Boa tarde a todos, começamos em dois minutos.               │
  │  Good afternoon everyone, we start in two minutes.           │
  ╰──────────────────────────────────────────────────────────────╯
```

### 91.2. Ecrã Captions na janela

```
┌──────────────────────────────────────────────────────────────────────┐
│ Captions                                            [ Start ]       │
│  Source  [ Microphone ] [ System audio ] [ Both ]                    │
│  Look    [ Size 18 ▾ ] [ Contrast high ] [ Position bottom ]         │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 00:12  Boa tarde a todos…                                      │  │
│  │        Good afternoon everyone…                                │  │
│  │ 00:19  …só falta o João.                                       │  │
│  │        …only João is missing.                                  │  │
│  └────────────────────────────────────────────────────────────────┘  │
│  Export  [ .srt ] [ .txt ]        Language pair  PT → EN             │
└──────────────────────────────────────────────────────────────────────┘
```

Regras: texto parcial em `muted` com itálico, texto confirmado em `ink`; nunca se apaga uma linha já mostrada, corrige-se no lugar com uma transição de 180 ms. Participantes aparecem com um número (`Speaker 1`) quando a diarização existe, e nunca com um nome inferido. Modo compacto mostra só a última linha traduzida; modo expandido mostra original e tradução.

## 92. Parlo Live

O Live não é um painel de tradução: é uma conversa em duas direções, com a voz de cada pessoa preservada. A interface tem três momentos distintos e um ecrã de configuração que não pode ser saltado às cegas.

### 92.1. Antes da chamada

```
┌──────────────────────────────────────────────────────────────────────┐
│ Live · set up the conversation                                       │
│                                                                      │
│  You            [ Portuguese (Portugal) ▾ ]   Voice: My voice ✓      │
│  The other      [ English ▾ ]                 Voice: not set —      │
│                                                     captions only    │
│                                                                      │
│  Call app       [ Zoom ▾ ]   ( detected: Zoom is running )           │
│  Microphone     [ Realtek Array ▾ ]    Output [ Speakers ▾ ]         │
│  Routing        [ Install virtual microphone ]  needs permission     │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ Two things will be true during the call: your words are         │  │
│  │ translated into English, and the other person hears them in     │  │
│  │ your voice only if your voice profile is ready for this pair.   │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  [ Start conversation ]              [ Use captions instead ]        │
└──────────────────────────────────────────────────────────────────────┘
```

Regras: a linha «Voice» diz sempre a verdade — `ready`, `not measured for this language`, `captions only`. Nenhuma aplicação de chamada é dada como compatível sem medição registada; a lista mostra as que foram testadas e as que estão por testar, em secções separadas. Instalar um dispositivo de áudio virtual exige consentimento explícito com o que vai ser criado e como é removido.

### 92.2. Durante a chamada, modo compacto

```
  ╭──────────────────────────────────────────────────────────────╮
  │  ◉ You speak          PT → EN        0.9 s     [ ▾ ]  [ ✕ ]  │
  │  A reunião fica para quinta.                                 │
  │  The meeting stays on Thursday.                              │
  ╰──────────────────────────────────────────────────────────────╯
```

Compacto mostra: quem fala, direção, latência, última frase nas duas línguas, e dois controlos (expandir, terminar). A latência é o valor real medido, com `warn` acima do objetivo do perfil e uma frase que diz o que está lento.

### 92.3. Durante a chamada, modo expandido

```
┌──────────────────────────────────────────────────────────────────────┐
│ Live · 12:04                                        [ Pause ]  [End] │
│                                                                      │
│  ◉ You — Portuguese                        ○ Ana — English           │
│     A reunião fica para quinta.               The meeting stays on    │
│     ▸ The meeting stays on Thursday.          Thursday.              │
│     voice: preserved · 0.9 s                  voice: preserved 1.1 s │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 12:04  You  PT  A reunião fica para quinta.                    │  │
│  │ 12:04  You  EN  The meeting stays on Thursday.                 │  │
│  │ 12:03  Ana  EN  Perfect, I will book the room.                 │  │
│  │ 12:03  Ana  PT  Perfeito, eu reservo a sala.                   │  │
│  └────────────────────────────────────────────────────────────────┘  │
│  Routing: virtual mic active · echo cancellation on                  │
│  Voice: yours in English, Ana's in Portuguese · measured, not claimed│
└──────────────────────────────────────────────────────────────────────┘
```

Regras: dois indicadores de participante, nunca mais, porque o produto suporta duas pessoas nesta fase. «voice: preserved» só aparece quando a identidade foi medida para aquele par; caso contrário lê-se `neutral voice, identified` e a interface di-lo em palavras. O histórico da chamada cresce por baixo e nunca empurra os controlos.

### 92.4. Depois da chamada

Ecrã de fecho com quatro linhas: duração, pares usados, latência mediana e falhas. Ações: guardar transcrição, exportar, apagar tudo. Guardar exige uma escolha explícita e explica onde fica o ficheiro. Apagar pede confirmação e lista o que desaparece.

## 93. Parlo Voice

```
┌──────────────────────────────────────────────────────────────────────┐
│ Voice                                                                │
│  A voice profile is how Parlo speaks in your voice instead of a      │
│  neutral one. Samples and everything made from them stay on this     │
│  computer.                                        [ New profile ]   │
│                                                                      │
│  ┌───────────────────────────┐  ┌───────────────────────────┐        │
│  │ My voice                  │  │ Ana (with permission)     │        │
│  │ PT-PT · consent recorded  │  │ PT-PT · consent recorded  │        │
│  │ Quality: good             │  │ Quality: fair             │        │
│  │ Used for: PT→EN, EN→PT    │  │ No engine supports it yet │        │
│  │ [ Play ] [ Open ] [ ⋯ ]   │  │ [ Play ] [ Open ] [ ⋯ ]   │        │
│  └───────────────────────────┘  └───────────────────────────┘        │
│                                                                      │
│  Advanced  ▸ Engines, embeddings and sample details                  │
└──────────────────────────────────────────────────────────────────────┘
```

Regras: cada cartão mostra nome, variante, estado de consentimento e qualidade em três níveis (`good`, `fair`, `noisy`), nunca uma percentagem. O menu `⋯` contém Revoke, Delete e Export consent record. «Advanced» abre o painel com motores compatíveis, representações existentes e a data da amostra; esse painel nunca aparece por omissão.

Fluxo de criação, em quatro passos e um ecrã por passo: explicar, declarar origem, gravar ou importar, ouvir e confirmar. Não existe botão «Skip consent»; existe «Cancel».

## 94. Parlo Meeting

### 94.1. Meeting Lite

```
┌──────────────────────────────────────────────────────────────────────┐
│ Meeting · recording                     [ New import ]  [ Search ]  │
│  ┌──────────────┬─────────────────────────────────────────────────┐  │
│  │ Sessions     │  Weekly sync · 42 min · 3 speakers              │  │
│  │ ● Weekly sync│  ┌───────────────────────────────────────────┐  │  │
│  │   Board 12/11│  │ 00:02:14  Speaker 1  Bom, vamos começar…  │  │  │
│  │   Interview  │  │ 00:02:31  Speaker 2  Sim, só uma nota…    │  │  │
│  │   Lecture    │  │ 00:03:02  Speaker 1  …                     │  │  │
│  │              │  └───────────────────────────────────────────┘  │  │
│  │              │  [ Rename speaker ] [ Export ] [ Delete ]       │  │
│  └──────────────┴─────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

### 94.2. Meeting Live

Durante a reunião o ecrã é uma faixa compacta com estado de captura, tempo, número de falantes e um botão de pausa; a transcrição cresce por baixo, em texto grande e legível de longe. O modo expandido acrescenta marcadores e pesquisa. Nunca há resumo em direto, porque um resumo a meio de uma reunião induz decisões erradas.

### 94.3. Meeting Chat

```
┌──────────────────────────────────────────────────────────────────────┐
│ Ask this meeting                                                      │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ What did we decide about the release date?                      │  │
│  └────────────────────────────────────────────────────────────────┘  │
│  The release moves to the 12th [00:14:22], because QA needs two      │
│  more days [00:15:01].                                               │
│                                                                      │
│  Sources   · 00:14:22 Speaker 2   · 00:15:01 Speaker 1               │
└──────────────────────────────────────────────────────────────────────┘
```

Regras: cada afirmação leva uma referência com timestamp, e clicar nela abre a transcrição nesse ponto. Sem referência, a resposta não é mostrada. O chat diz «not in this recording» quando não encontra, em vez de responder de memória.

## 95. Model Hub

```
┌──────────────────────────────────────────────────────────────────────┐
│ Models                                            [ Import a model ] │
│  Recommended for this computer                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ Parakeet TDT 0.6B v3   Recognition PT-PT   600 MB  installed   │  │
│  │ Whisper small          Recognition fallback 480 MB  installed  │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  Categories   [ All ] [ Recognition ] [ Translation ] [ Voice ]      │
│  Language     [ Portuguese (Portugal) ▾ ]   [ Search… ]              │
│                                                                      │
│  ┌───────────────────────────┐  ┌───────────────────────────┐        │
│  │ OPUS-MT PT→EN             │  │ Chatterbox PT-PT          │        │
│  │ Translation · 310 MB      │  │ Voice cloning · 4.3 GB    │        │
│  │ candidate · not validated │  │ experimental · needs GPU  │        │
│  │ [ Install ]               │  │ [ Details ]               │        │
│  └───────────────────────────┘  └───────────────────────────┘        │
│  Storage 4 models · 2.3 GB used · 18 GB free      [ Clean up ]       │
└──────────────────────────────────────────────────────────────────────┘
```

Regras: o primeiro bloco é sempre a recomendação do PIMO e não uma lista; o utilizador normal não precisa de abrir mais nada. Cada cartão mostra um estado em palavra (`installed`, `candidate`, `experimental`, `incompatible`) e nunca uma pontuação. Detalhes abrem painel lateral com licença do código, licença dos pesos, tamanho, requisitos, versão e hash. Instalar mostra o tamanho, a origem e a licença antes de começar, e o progresso é por ficheiro, cancelável.

## 96. Definições

```
┌──────────────────────────────────────────────────────────────────────┐
│ Settings                                                             │
│  General        Language spoken      [ Portuguese (Portugal) ▾ ]     │
│  Languages      Write in             [ English ▾ ]                   │
│  Voice          Start with Parlo   [ on ]                            │
│  Audio          Close to tray      [ on ]                            │
│  Shortcuts      Appearance         ( System ) ( Light ) ( Dark )     │
│  Appearance                                                          │
│  Privacy        ▸ Advanced                                           │
│  History                                                             │
│  Models                                                              │
│  Licences                                                            │
│  Diagnostics                                                         │
└──────────────────────────────────────────────────────────────────────┘
```

Grupos, na ordem: General, Languages, Voice, Audio and devices, Shortcuts, Appearance, Privacy, History and storage, Models (advanced), Licences and open source, Diagnostics. Cada grupo tem no máximo oito controlos visíveis; o resto vive em «Advanced» dentro do grupo. «Privacy» mostra o estado real: microfone, captura de áudio do sistema, perfis existentes, última eliminação, e um botão «Delete everything» que lista o que apaga antes de apagar.

Atalhos: lista com o atalho actual, um botão «Change», deteção de conflito com o sistema e reposição para o valor por omissão. Valores por omissão: ditado `Ctrl+Alt+D`, legendas `Ctrl+Alt+C`, live `Ctrl+Alt+L`, pausa `Ctrl+Alt+Space`.

## 97. Onboarding

Cinco ecrãs, nenhum obrigatório depois do segundo. Cada um tem uma frase de contexto, uma decisão e um caminho de saída.

```
┌──────────────────────────────────────────────────────────────────────┐
│  ●                                                                   │
│  Speak Portuguese. Be heard in English.                              │
│                                                                      │
│  Parlo listens on this computer, translates, and speaks the other    │
│  language back. Audio never leaves the machine, and there is no      │
│  account and no telemetry.                                           │
│                                                                      │
│  [ Continue ]                                    [ I know Parlo ]   │
└──────────────────────────────────────────────────────────────────────┘
```

1. **O que é**: a frase acima e um botão. Duração alvo: 15 segundos.
2. **Idiomas**: idioma falado e idioma de escrita, com deteção do sistema como sugestão.
3. **Microfone**: teste de 3 segundos com medidor e resultado em palavras (`clear`, `quiet`, `blocked`), com instruções concretas quando falha.
4. **Modelos**: duas recomendações (equilibrada e leve) com tamanho, espaço livre e o que cada uma permite; uma única confirmação para instalar.
5. **Primeira frase**: ditar uma frase e ver a tradução, com a opção de criar Voice Profile mais tarde. Termina com o atalho à vista.

Regras: progresso visível (`1 of 5`); nenhum passo bloqueia os seguintes excepto o download; tudo o que foi decidido pode ser alterado depois em Definições; nunca se pede conta, email ou telefone; a criação de Voice Profile não faz parte do onboarding obrigatório.

## 98. Mapa de ecrãs e fluxos

```
                       ┌───────────────┐
        ┌──────────────┤   Main window │
        │              └───────┬───────┘
        │                      │
   ┌────▼────┐  ┌──────┐  ┌────▼────┐  ┌───────┐  ┌────────┐
   │  Home   │  │Dictate│ │Captions │  │ Live  │  │ Voice  │
   └────┬────┘  └───┬──┘  └────┬────┘  └───┬───┘  └───┬────┘
        │           │          │           │          │
        │      ┌────▼────┐ ┌───▼─────┐ ┌───▼──────┐ ┌─▼─────┐
        │      │ Island  │ │ Caption │ │ Live bar │ │Consent│
        │      │ (always │ │  bar    │ │ compact  │ │ flow  │
        │      │  on top)│ │         │ │ expanded │ │       │
        │      └─────────┘ └─────────┘ └───┬──────┘ └───────┘
        │                                  │
        │                            ┌─────▼─────┐
        │                            │ Summary   │
        │                            └───────────┘
   ┌────▼─────┐  ┌──────────┐  ┌───────────┐  ┌──────────┐
   │ History  │  │ Meeting  │  │Model Hub  │  │ Settings │
   └──────────┘  └──────────┘  └───────────┘  └──────────┘
```

Fluxo de ditado: atalho → ilha a ouvir → atalho outra vez → escrita → clipboard → ilha diz «COPIED» e fica visível 4 s. Fluxo de live: Live → configuração → verificação de voz e encaminhamento → conversa compacta → expandida se o utilizador quiser → resumo → guardar ou apagar.

## 99. Responsividade, monitores e plataformas

| Situação | Regra |
|---|---|
| Janela 1 180 × 820 (por omissão) | Barra lateral expandida, área de trabalho com margem de 32 px |
| Janela 900 × 620 (mínima) | Barra lateral colapsada a 64 px, cartões a uma coluna, tabelas com 3 colunas |
| Janela abaixo de 900 de largura | Não é permitida; a janela pára no mínimo e a área de trabalho faz scroll |
| Escala de sistema a 125% e 150% | Todos os tamanhos são lógicos; a ilha cresce com a escala e mantém a margem de 64 px |
| Dois monitores | A ilha nasce no monitor do cursor e memoriza a posição por monitor; a janela principal memoriza por monitor |
| Monitor de 3 840 × 2 160 | Largura máxima de leitura de 1 040 px no texto corrido; o resto é espaço |
| Windows | Plataforma inicial; ilha e atalho global já implementados; encaminhamento por aplicação depende de dispositivo virtual medido |
| macOS | Janela e ilha previstas; o atalho global exige implementação própria; permissões de microfone e de acessibilidade declaradas ao utilizador |
| Linux | X11 e Wayland tratados em separado; a ilha exige camada de sobreposição disponível; sem promessa até existir medição |

Regra de honestidade por plataforma: um ecrã que não funciona numa plataforma diz isso dentro do próprio ecrã, com a razão, e não desaparece deixando o utilizador à procura.

## 100. Acessibilidade visual e de teclado

- Contraste mínimo 4,5:1 em texto normal e 3:1 em texto grande e contornos; o tema escuro é verificado com a mesma tabela do tema claro.
- Foco visível em todos os elementos interativos: anel de 2 px `accent` com 2 px de afastamento, nunca removido sem substituto.
- Ordem de tabulação igual à ordem visual; nenhum `tabindex` positivo.
- Toda a ação da janela principal tem atalho de teclado documentado no próprio ecrã, via tooltip ou menu.
- Leitores de ecrã: marcos (`main`, `nav`, `aside`), títulos hierárquicos, `aria-live` nos estados, `aria-label` em todos os botões de ícone, e o estado de gravação anunciado uma vez por transição, não continuamente.
- Legendas com tamanho de 14 a 32 px, opção de alto contraste e fundo opaco, escolhidas no ecrã de legendas.
- Nenhuma informação depende só de cor: os badges têm palavra, os erros têm frase, o medidor de áudio tem um valor em dB quando pedido.
- Alvos de toque e rato com 32 px mínimos e 44 px nas ações principais.
- `prefers-reduced-motion` desliga transições e pulsos; `prefers-contrast: more` aumenta o contraste das bordas e remove transparências.
- Textos da interface em inglês, com chaves de tradução desde o início; o idioma da interface é independente do idioma falado e do idioma de escrita.

## 101. Regras de produto que a interface não pode violar

1. O caminho normal é automático: o PIMO escolhe, o utilizador confirma o download e fala.
2. Nenhum ecrã do fluxo normal pede para comparar modelos, benchmarks ou motores.
3. A janela principal nunca é necessária para ditar, ver legendas ou ver o estado de uma sessão Live.
4. A voz de uma pessoa nunca é substituída por uma voz neutra sem o dizer em palavras, no mesmo ecrã, antes de o áudio tocar.
5. Ações destrutivas pedem confirmação com o nome concreto do que desaparece.
6. Um download diz tamanho, origem, licença e espaço livre antes de começar.
7. A privacidade local é visível onde é relevante, não escondida em Definições.
8. Nenhum ecrã mostra uma métrica que não ajude a decisão imediata.
9. Nenhum estado fica sem saída: erro tem sempre uma ação e um caminho alternativo identificado.
10. O que está por validar aparece marcado como tal na interface, com a palavra `candidate` ou `experimental`, e nunca com um selo de qualidade.
