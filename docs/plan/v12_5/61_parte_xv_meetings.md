# PARTE XV · REUNIÕES, MEMÓRIA E SELEÇÃO DE MODELOS

Esta parte acrescenta três decisões de produto que estavam ausentes ou incompletas: a política de seleção de modelos deixa de poder ser lida como bloqueio, as reuniões sobem a pilar principal e ganham memória entre sessões, e a hierarquia final do produto passa a ter quatro pilares em vez de dois. Não substitui nenhuma parte anterior. A Parte X (secção 66) descrevia Meeting Lite, Live e Chat como três modos; aqui o Meeting passa a ser uma experiência completa, e a secção 66 mantém-se como especificação dos modos de captura e de leitura.

> **O que está implementado hoje: nada desta parte.** Não existe sessão de reunião, transcrição guardada, separação de falantes, resumo, Ask Parlo nem memória. O que existe é a captura de áudio do Live, a transcrição do motor e o histórico de traduções. Tudo o resto é especificação, com as camadas da secção 113 a dizer em que ordem pode ser construído.

## 103. Seleção de modelos: recomenda, não bloqueia

### 103.1. O princípio

> **O Parlo recomenda. O utilizador decide.**

O PIMO continua a ser a experiência padrão: em nenhum fluxo normal se pede a alguém que compare modelos, quantizações ou fornecedores. Mas a recomendação é um ponto de partida, nunca uma vedação. Qualquer modelo compatível pode ser escolhido, mesmo que seja pior em qualidade, latência ou consumo.

Isto corrige uma leitura possível da Parte VI: as eliminações da secção 29 são **técnicas** (arquitetura errada, sistema operativo errado, licença que impede distribuição, memória insuficiente para arrancar), não uma ordenação de gostos. Um modelo que arranca e produz resultado nunca é escondido do utilizador avançado.

### 103.2. O que a recomendação analisa

| Sinal | Uso na decisão | Quem mede |
|---|---|---|
| CPU, núcleos, instruções | Escolha de threads e de quantização | Perfil do dispositivo, secção 30 |
| GPU e VRAM | Elegibilidade de backends acelerados | Perfil do dispositivo |
| RAM disponível | Tamanho máximo de modelo carregável | Perfil do dispositivo |
| Sistema operativo e arquitetura | Binários e runtimes disponíveis | Manifesto do modelo |
| Idioma e variante | Cobertura declarada e validada | Manifesto e benchmarks |
| Tarefa | Conjunto de modelos candidatos | Catálogo, secção 20 |
| Prioridade qualidade/latência | Perfil de desempenho, secção 58 | Preferência do utilizador |
| Utilização recente | Evitar trocas sem motivo | Monitorização, secção 35 |

O resultado é uma configuração, não um número: um modelo por tarefa, com quantização, backend, threads e voz. A interface mostra-a com a frase **«O Parlo escolheu a melhor configuração disponível para este computador e esta tarefa.»** e uma ligação discreta para o detalhe.

### 103.3. Escolha manual

No **Advanced / Model Hub** existem, sempre, as mesmas quatro vistas:

| Vista | Conteúdo | Ação permitida |
|---|---|---|
| Recomendado | Configuração atual do PIMO e a razão em uma frase | Aceitar, ou abrir detalhe |
| Instalados | O que está no disco, com espaço real por modelo | Usar, substituir, remover, reverter |
| Disponíveis | O que o catálogo oferece e é compatível | Instalar |
| Avançado | Versões, quantizações, fornecedores, requisitos, licença, notas | Escolher manualmente |

Cada cartão mostra qualidade observada, latência observada com condições, requisitos de hardware, tamanho, licença e estado de validação. Um modelo sem medição diz «por medir», nunca um número inventado.

### 103.4. Avisar sem impedir

| Situação | O Parlo faz | O Parlo não faz |
|---|---|---|
| Modelo compatível pior que o recomendado | Avisa em uma frase e permite continuar | Bloquear o botão |
| Modelo compatível mas não validado | Marca «compatível, não validado» e explica o que falta | Apresentar como equivalente |
| Modelo com maior latência que o perfil | Avisa que a conversa pode ficar mais lenta | Impedir a escolha |
| Modelo experimental | Exige confirmação e registra a escolha | Esconder a opção |
| Modelo incompatível com o hardware | Mostra o motivo técnico e a alternativa mais próxima | Instalar e falhar depois |
| Modelo com licença que impede distribuição | Não aparece no catálogo normal; aparece em «externo» com o aviso | Instalar silenciosamente |

A frase de aviso é sempre a mesma em todo o produto: **«Este modelo pode dar menor qualidade ou maior latência do que a configuração recomendada. Continuar?»**

### 103.5. O que a escolha manual não pode quebrar

- Uma escolha manual fica registada por tarefa e por idioma, e aparece no Home enquanto estiver ativa, para não haver dúvida sobre o que está a correr.
- «Voltar à configuração automática» existe em todos os ecrãs onde há escolha manual, num clique.
- A proteção da identidade vocal da secção 36 não é uma preferência: se um modelo escolhido à mão não consegue preservar a voz, o aviso é obrigatório e a alternativa é oferecida antes de a sessão começar.
- Um modelo que falha duas vezes seguidas volta ao recomendado com uma frase na interface, e o utilizador pode insistir de novo depois.

### 103.6. Critérios de aceitação

1. Nenhum fluxo normal exige escolher modelo.
2. Todos os modelos compatíveis são alcançáveis a partir do Advanced.
3. Toda a escolha manual é reversível num clique.
4. Nenhum aviso de qualidade é apresentado como bloqueio.
5. Nenhuma incompatibilidade é apresentada como preferência.

## 104. Parlo Meetings como pilar

### 104.1. Definição

**Meetings** é a experiência completa de uma reunião: entrar, ouvir traduzido, transcrever, identificar quem fala, perceber o que ficou decidido e poder voltar a perguntar mais tarde. O Live é um componente dentro desta experiência, não o produto inteiro.

> **A reunião não termina quando a chamada termina.**

### 104.2. O que uma sessão contém

| Elemento | Durante | Depois |
|---|---|---|
| Tradução | Sim, quando configurada | Só o texto |
| Transcrição | Incremental | Completa e pesquisável |
| Falantes | Identificados por marcador | Nomeados quando há evidência |
| Decisões | Marcadas à medida que surgem | Lista com evidência |
| Action items | Marcados à medida que surgem | Lista com responsável, prazo e contexto |
| Resumo | Não | Gerado no fim |
| Perguntas | Não | Ask Parlo, sobre esta reunião |
| Memória | Não | Indexada para perguntas futuras |

### 104.3. Reutilização obrigatória

Meetings não duplica infraestrutura. Cada componente tem um dono e é partilhado com o Live e com o Dictate.

| Componente | Dono | Meetings usa |
|---|---|---|
| Captura de áudio | Núcleo, secção 58 | O mesmo caminho do Live |
| Encaminhamento e dispositivo virtual | Live, secção 66.2 | O mesmo, sem segunda implementação |
| Reconhecimento | Motor, secção 16 | O mesmo, com legendas estáveis |
| Tradução | Motor, secção 17 | A mesma |
| Voz | Parte II | A mesma, com a mesma proteção de identidade |
| Diarização e VAD | Secção 19 | Camada adicional, opcional |
| Gestão de sessão | Núcleo | Estados e recuperação iguais |
| Armazenamento | Local, secção 61 | Pasta de dados do utilizador |

Uma reunião é, portanto, **Live + transcrição persistida + processamento de falantes + inteligência pós-reunião**, e não um segundo produto com o seu próprio áudio.

## 105. Durante a reunião

### 105.1. Transcrição incremental

As palavras aparecem enquanto são ditas, com três estados iguais aos do Live: parcial, estabilizado e final. O texto parcial nunca é guardado no transcript; só o estabilizado e o final o são. Se o motor reescrever uma frase, a versão guardada é a última estabilizada, e a interface não mostra correções a tremer.

### 105.2. Separação de falantes

Enquanto não há nomes, os participantes são **Speaker 1**, **Speaker 2**, **Speaker 3**. A diarização é uma inferência acústica: diz «vozes diferentes», não diz «pessoas diferentes». Por isso o produto nunca apresenta a separação como identificação de uma pessoa, e qualquer fusão ou divisão errada tem de ser corrigível à mão, com o transcript a manter o segmento original para auditoria.

### 105.3. Nomeação de falantes e incerteza

Quando a conversa contém evidência suficiente, o Parlo pode propor um nome. A evidência típica é uma saudação com nome próprio dirigida a quem responde a seguir.

```text
Jorge: Olá, tudo bem, Miguel?
Speaker 2: Sim.
```

O sistema propõe `Speaker 2 → Miguel` e mostra a proposta como proposta, não como facto:

- confiança alta e evidência direta: o nome é aplicado e fica marcado como inferido, com a frase que serviu de prova;
- confiança média: aparece como sugestão com um botão de confirmação;
- sem evidência: continua `Speaker 2`, e nada é inventado.

Nunca se atribui um nome por semelhança de voz a uma pessoa conhecida. Não há reconhecimento biométrico de terceiros nesta versão, e a secção 62 continua a impedir que um perfil de voz seja usado como identificador.

### 105.4. Identidade retroativa

A nomeação é retroativa por desenho: quando `Speaker 2` passa a `Miguel`, todos os segmentos anteriores desse marcador passam a mostrar `Miguel` no transcript final.

```text
Antes:  Speaker 2: Sim.
        Speaker 2: Posso tratar disso amanhã.

Depois: Miguel: Sim.
        Miguel: Posso tratar disso amanhã.
```

Cada segmento guarda o marcador original e o nome aplicado, com data e origem da alteração. Assim o transcript fica limpo para leitura e continua auditável se alguém quiser perceber porque é que dois segmentos passaram a ter o mesmo nome. Uma nomeação errada desfaz-se num clique e volta ao marcador.

### 105.5. O que o ecrã mostra

Durante a chamada, só o que serve a conversa: quem está a falar, o que foi dito, o que foi traduzido, quantos participantes e três estados de saúde (tradução, transcrição, voz). A secção 111 fixa a disposição.

## 106. Depois da reunião: Meeting Summary

### 106.1. Primeira classe, não uma notificação

O fim de uma reunião nunca produz a frase «Reunião guardada». Produz uma superfície de resultado com o que interessa na primeira leitura.

```text
┌──────────────────────────────────────────────┐
│ Meeting complete                             │
│ Product Sync · 1 h 12 min · 4 participants   │
├──────────────────────────────────────────────┤
│ Summary                                      │
│ ...duas a cinco frases...                    │
│                                              │
│ 3 Decisions · 2 Action items · 1 Open question│
│                                              │
│  [ What was decided? ]  [ View transcript ]  │
└──────────────────────────────────────────────┘
```

### 106.2. Blocos do resultado

| Bloco | Conteúdo | Regra |
|---|---|---|
| Summary | O que foi a reunião em poucas frases | Sem opinião, sem promessas |
| Participants | Quem falou, com marcador e nome quando houver | Nome inferido é marcado como inferido |
| Topics | 3 a 8 assuntos, com o momento em que começaram | Ordem cronológica |
| Decisions | Só o que foi fechado | Ver 106.3 |
| Action items | Responsável, tarefa, prazo, contexto, momento | Ver 106.4 |
| Open questions | O que ficou sem resposta | Sem sugestões disfarçadas |
| Transcript | Texto completo, pesquisável, com saltos por timestamp | Nunca cortado |
| Highlights | 3 a 6 momentos relevantes | Ligados ao texto, não a resumos |

### 106.3. Proposta não é decisão

> «Acho que podíamos lançar sexta-feira.» → **Proposal / discussed**

Só quando existe fecho explícito o item passa a decisão.

| Enunciado | Classificação | Porquê |
|---|---|---|
| «Acho que podíamos lançar sexta.» | Proposta | É uma opinião, sem fecho |
| «Então fica sexta-feira.» | Decisão | Fecho explícito sobre a mesma proposta |
| «Vou ver isso amanhã.» | Action item | Compromisso pessoal com prazo |
| «Talvez fosse melhor outra cor.» | Discussão | Sem compromisso |
| «Ficou combinado: sexta.» | Decisão | Fecho com confirmação |
| «Não sei, decidimos depois.» | Open question | Explicitamente em aberto |

Cada decisão guarda a frase que a fechou e o momento em que foi dita. Se a mesma proposta aparecer em duas reuniões, a mais recente com fecho prevalece, e a anterior fica visível no histórico como substituída — nunca apagada.

### 106.4. Action items

Um action item tem responsável, tarefa e, quando o texto o permite, prazo e contexto. Quando o responsável não é claro, fica «por atribuir» em vez de um nome adivinhado. Quando o prazo é relativo («até sexta»), guarda a frase original e a data calculada, para se poder verificar a conta.

### 106.5. Evidência

Todo o item do resumo aponta para o momento que o sustenta: timestamp, falante e frase. Sem evidência, um item não entra no resumo; aparece, se for caso disso, em «possivelmente relevante», separado e identificado.

## 107. Ask Parlo sobre uma reunião

### 107.1. Perguntas suportadas

Perguntas em linguagem natural, sobre a reunião aberta: o que ficou decidido, quem ficou responsável por quê, o que ficou pendente, quando se lança, quem discordou, porquê, resumo dos últimos quinze minutos, o que é que uma pessoa ficou de fazer.

### 107.2. Respostas com prova

Cada resposta cita os segmentos em que se baseia, com falante e timestamp, e permite saltar para o momento no transcript. Uma resposta sem segmentos de suporte não é dada como resposta.

### 107.3. Recusa honesta

> «Não encontro isso nesta reunião. Posso mostrar as decisões registadas ou procurar nas reuniões anteriores.»

Três regras: uma inferência é sempre escrita como inferência; um nome incerto aparece como «Speaker 2 (possivelmente Miguel)»; e o que não está no transcript não é respondido a partir de conhecimento geral.

### 107.4. Sem modelo remoto por omissão

O Ask Parlo responde sobre os dados locais. Se uma resposta exigir processamento externo, isso é dito antes, com o que sairia da máquina, e é preciso consentimento; sem consentimento, responde o caminho local, com menos capacidade e a limitação declarada.

## 108. Meeting Memory

### 108.1. O que é

Uma camada local de memória sobre reuniões passadas. Não é uma pilha de resumos: um resumo perde quem disse o quê, quando e com que prova. A memória guarda a estrutura.

| Camada | Guarda | Serve para |
|---|---|---|
| Transcript | Segmentos com falante, tempo e texto | Prova e citação |
| Pessoas | Nomes, marcadores e ligações entre reuniões | «O que ficou com o Jorge?» |
| Decisões | Enunciado, evidência, reunião, data | «Quando decidimos isto?» |
| Action items | Tarefa, responsável, prazo, estado | «O que está pendente?» |
| Tópicos | Assunto, reuniões onde apareceu | «Quando falámos disto pela primeira vez?» |
| Relações | Substituições, continuações, contradições | «O que mudou desde a última?» |

### 108.2. Perguntas entre reuniões

```text
Quando decidimos isto?
O que ficou combinado com o Jorge?
O que ficou pendente da última reunião?
Que tarefas ficaram atribuídas ao Miguel?
Quando falámos pela primeira vez deste problema?
Quais foram as decisões sobre o lançamento nas últimas três reuniões?
O que mudou desde a reunião anterior?
```

### 108.3. Proveniência obrigatória

Toda a resposta de memória diz de que reunião ou reuniões veio, com data, e permite abrir o momento. Uma resposta que junta três reuniões cita as três. Se duas reuniões se contradizem, a resposta mostra as duas e diz qual é a mais recente, em vez de escolher em silêncio.

### 108.4. Limites técnicos declarados

- A indexação é local, sobre o transcript guardado; não há envio de conteúdo para índices externos.
- A pesquisa é textual e semântica sobre dados locais; a qualidade depende da transcrição, e uma transcrição má produz uma memória má.
- A memória não é uma base de conhecimento pessoal genérica: responde sobre reuniões que o Parlo processou.
- Sem reuniões guardadas, a memória não tem o que responder, e diz isso.

## 109. Privacidade e controlo da memória

### 109.1. Onde vive cada dado

| Dado | Local | Formato | Eliminável |
|---|---|---|---|
| Transcript | Pasta de dados do utilizador | Texto com metadados por segmento | Sim, por reunião ou tudo |
| Resumo e itens | Pasta de dados, por reunião | Estruturado, com referências | Sim |
| Identidades de falante | Pasta de dados | Nome, marcador, evidência | Sim, por pessoa |
| Índice de memória | Pasta de dados | Derivado, reconstruível | Sim, reconstruível do transcript |
| Áudio da reunião | Não guardado por omissão | — | Opcional, com consentimento |

Nenhum destes dados sai da máquina no modo local. Não há telemetria sobre reuniões, nem contagem de reuniões, nem nomes de participantes em qualquer pedido externo. A verificação de atualizações continua a ser a única chamada permitida e não transporta nada disto.

### 109.2. Eliminação

- **Apagar uma reunião:** transcript, resumo, itens, áudio se existir e entradas de índice; a operação diz quantos ficheiros removeu.
- **Apagar toda a memória:** todas as reuniões e o índice, com confirmação escrita e a frase do que se perde.
- **Apagar uma pessoa:** o nome é retirado de todos os segmentos e mantém-se o marcador, para o transcript não ficar com buracos de conteúdo.
- **Desfazer uma nomeação:** volta ao marcador sem tocar no texto.
- A eliminação respeita os limites técnicos documentados na secção 61: uma cópia em backup do sistema operativo não é alcançável pelo Parlo, e isso é dito, não escondido.

### 109.3. Processamento externo

Se alguma vez existir um fornecedor externo para resumo ou pesquisa, a arquitetura exige: escolha explícita, indicação do que sai (texto, nunca áudio por omissão), indicação do que fica, e a possibilidade de manter tudo local. Nada disto está implementado nem prometido nesta versão.

## 110. Integrações de reunião e níveis de suporte

| Plataforma | Estado hoje | O que falta para subir de nível |
|---|---|---|
| Zoom | Não testado | Captura e encaminhamento medidos, numa versão concreta |
| Microsoft Teams | Não testado | O mesmo, com dispositivo virtual |
| Google Meet | Não testado | O mesmo, em navegador |
| Discord | Não testado | O mesmo, com dispositivo virtual |
| WhatsApp | Não testado | Chamadas de voz, com dispositivo virtual |
| Outras | Sem suporte declarado | Nada prometido |

Quatro estados e nada entre eles: **Supported** (funciona e está documentado), **Tested** (foi medido numa versão e numa máquina, com resultado registado), **Experimental** (funciona em condições limitadas, com aviso), **Unavailable** (não funciona, e o produto diz isso). Nenhuma plataforma sobe de estado sem medição registada em `docs/BENCHMARKS.md`, e nenhuma lista de aplicações compatíveis aparece no README antes disso.

## 111. UI de Meetings

### 111.1. Durante a chamada

A superfície é pequena, arrastável e não rouba foco. Não tem modelos, quantização, VRAM, caminhos nem registos.

```text
┌──────────────────────────────────┐
│ ● Parlo Meeting              ⋯  │
├──────────────────────────────────┤
│ Português → English              │
│                                  │
│ A falar: Miguel                  │
│ «...e por isso prefiro sexta.»   │
│                                  │
│ 4 participantes                  │
│ Tradução      ● ativa            │
│ Transcrição   ● ativa            │
│ Voz           ● a preservar      │
│                                  │
│    [ Pausar ]      [ Terminar ]  │
└──────────────────────────────────┘
```

### 111.2. Expandida

Quando alguém quer acompanhar, a superfície cresce para duas colunas: original e traduzido, com o falante à esquerda, marcadores de tempo e a lista de participantes. As opções continuam escondidas atrás de um menu, e nenhuma opção destrutiva fica a um clique acidental.

### 111.3. Pós-reunião

Três separadores: **Resumo**, **Transcript** e **Perguntar**. O resumo mantém a ordem da secção 106.2; o transcript tem pesquisa, filtro por falante e saltos; o perguntar é uma conversa com referências. Fechar a janela não apaga nada, e apagar exige confirmação.

### 111.4. Acessibilidade

Legendas disponíveis sempre que a voz não possa ser reproduzida; contraste e tamanho de texto iguais aos do Live; navegação completa por teclado; leitura de ecrã a anunciar mudança de falante sem repetir cada palavra parcial; e nenhum estado comunicado só por cor.

## 112. Hierarquia de produto final

| Pilar | Promessa | Estado |
|---|---|---|
| Dictate | Falar e ver escrito, no idioma falado ou noutro | Implementado, com tradução de ditado |
| Live | Falar e ser ouvido noutra língua, em tempo real | Parcial: um sentido, com voz neutra |
| Meetings | Reunião traduzida, transcrita e compreendida | Especificado |
| Meeting Memory | Perguntar e recuperar o que aconteceu | Especificado |

As restantes capacidades servem estes quatro: Perfis de Voz, Encaminhamento de Áudio, Integrações, Histórico, Gestão de Modelos, Seleção Automática, Definições, Diagnóstico e Avançado. Nenhuma delas é um quinto pilar, e nenhuma aparece ao mesmo nível dos quatro no ecrã inicial.

> **O Parlo não deve apenas processar o que foi dito. Deve ajudar a recuperar o que aconteceu.** Durante a conversa entende; depois organiza; mais tarde recorda.

## 113. Implementação por camadas

### 113.1. Camada 1, o mínimo que já é útil

- Sessão de reunião com início, pausa, fim e recuperação depois de falha.
- Captura de áudio pelo caminho do Live.
- Transcrição incremental persistida por segmento.
- Separação de falantes com marcadores genéricos.
- Nomeação básica por evidência direta, com confirmação.
- Transcript completo e pesquisável.
- Resumo, decisões, action items e questões em aberto, todos com evidência.
- Ask Parlo sobre a reunião aberta.

Critério de conclusão: uma reunião de trinta minutos em português produz um transcript legível, entre três e oito tópicos, pelo menos uma decisão ou a frase de que não houve nenhuma, e duas perguntas respondidas com citação correta.

### 113.2. Camada 2, identidade e evidência

- Nomeação retroativa robusta, com desfazer e histórico.
- Ligações de evidência em todos os itens.
- Distinção fina entre proposta, discussão, decisão e questão aberta.
- Melhor separação de falantes com sobreposição de vozes.
- Pesquisa dentro de uma reunião por pessoa, decisão e tarefa.

### 113.3. Camada 3, memória

- Índice local entre reuniões.
- Perguntas sobre várias reuniões, com proveniência por reunião.
- Relações entre pessoas, decisões e tarefas, incluindo substituições e contradições.
- Eliminação selectiva por pessoa e reconstrução do índice.

### 113.4. Dependências

```text
Live (captura, reconhecimento, tradução)
  └─ Sessão de reunião e transcript
       ├─ Falantes e nomeação
       ├─ Resumo, decisões e action items
       │    └─ Ask Parlo sobre uma reunião
       └─ Índice local
            └─ Memória entre reuniões
```

A memória depende do transcript; o transcript depende de uma sessão que já saiba persistir; a sessão depende da captura que o Live já usa. Por isso a ordem não é negociável: não se começa por um índice de pesquisa sobre uma transcrição que ainda não existe.

### 113.5. O que nunca entra nesta parte

- Resumo apresentado como acta oficial.
- Nome de pessoa atribuído por semelhança de voz.
- Reuniões enviadas para fora da máquina sem escolha explícita.
- Contagem de participantes, nomes ou conteúdo em qualquer pedido de rede.
- Promessa de compatibilidade com uma aplicação de chamadas sem medição.
