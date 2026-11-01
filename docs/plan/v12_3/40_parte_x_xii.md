# PARTE X · QUALIDADE, ROADMAP E FASES

## 55. Lições de Handy, VoiceStudio e Wispr Flow

| Projeto | O que faz bem | O que o Parlo aprende | O que não copia |
|---|---|---|---|
| Handy | Fluxo simples, atalho, VAD, escolha entre Whisper e Parakeet, inserção no campo ativo | Clareza de fluxo e custo baixo de instalação | A ausência de gestão de modelos e de voz |
| VoiceStudio | Gestor de modelos extensível, deteção de dispositivo, instalação sob confirmação | Catálogo amplo e instalação explícita | A exposição da complexidade ao utilizador |
| Wispr Flow | Ditado com tradução e experiência polida | Barra flutuante discreta e inserção fiável | Dependência de serviço externo |

Nenhuma destas comparações afirma superioridade. O Parlo pode ser melhor na combinação de voz própria, execução local e governação de licenças; e pode ser pior em maturidade, suporte de plataformas e polimento durante muito tempo.

## 56. Roadmap incremental

Cada release é instalável, útil por si só e medida. Os números de versão são sequenciais a partir da última tag publicada e não correspondem a marcos internos.

| Release | Nome | O que o utilizador passa a poder fazer | Gate de saída |
|---|---|---|---|
| Próxima | Tradução fechada | Traduzir PT↔EN sem dependência AGPL, com par aprovado | G-MT: nenhum par declarado sem corpus representativo |
| Seguinte | Voz própria | Criar Voice Profile e ouvir uma tradução na própria voz | G-Voice: consentimento, qualidade e identidade medidos |
| Seguinte | Modelos | Instalar, trocar e remover modelos com licença e hash visíveis | G-Models |
| Seguinte | Automático | Auto Select e fallback a funcionar em todas as tarefas | G-Select |
| Seguinte | Reuniões Lite | Importar gravação e obter transcrição com falantes | G-Meeting-Lite |
| Depois | Live | Conversa bidirecional com voz | G-Live |
| Depois | Plataforma | API estável, SDK e plugins | G-Platform |

Regra: as releases só avançam quando os gates fecham. Não há calendário neste documento, e uma release pode demorar o tempo que o gate exigir.

## 57. Fases de execução e critérios de conclusão

| Fase | Trabalho | Critério de conclusão |
|---|---|---|
| 1 | Resolver a tradução EN→PT | Checkpoint redistribuível, licença confirmada, conjunto de 100 frases com números e negações acima do limiar, sem regressões |
| 2 | Remover o Argos do núcleo | Nenhuma dependência AGPL no caminho principal; catálogo sem artefactos não verificados |
| 3 | Voice Profile e consentimento | Criar, ouvir, exportar e apagar um perfil; registo de consentimento completo |
| 4 | Integrar clonadores | Qwen3-TTS Base e Chatterbox PT-PT medidos em CPU e GPU, com identidade e latência registadas |
| 5 | Proteção de dados vocais | Encriptação opcional, eliminação total verificada, nenhuma escrita fora da pasta de dados |
| 6 | Manifestos completos | Todos os campos da secção 28 preenchidos para cada candidato |
| 7 | Model Hub | Instalar, atualizar, remover, verificar hash e explicar escolha |
| 8 | Auto Select e fallback | Cadeias da secção 36 aplicadas, com aviso e registo |
| 9 | Benchmarks internos | Corpus versionado, execução reproduzível e resultados publicados com condições |
| 10 | Documentação e transparência | Privacidade, licenças, voz, modelos e limitações publicados e coerentes com o código |

As fases 1 a 3 têm precedência. A clonagem antes da voz neutra é ordem de produto, não preferência: sem perfil de voz e sem tradução fiável, a clonagem amplifica um erro em vez de criar valor.

## 58. Definição de concluída e regras de corte

Uma release está concluída quando: instala em máquina limpa e atualiza sobre a anterior; o rollback funciona; o benchmark do corpus está atualizado com hardware e versões; a matriz de licenças foi revista para tudo o que a release inclui; a auditoria de rede mostra zero ligações externas; o changelog declara limitações conhecidas; a acessibilidade mínima foi verificada nos fluxos principais; cada funcionalidade está classificada como experimental, preview, beta ou estável; e pelo menos cinco utilizadores externos a experimentaram.

Regras de corte: se o gate falha, a release não sai e a anterior continua a ser a estável; se um modelo não passa, a release sai sem ele com a alternativa já aprovada; trabalho fora do objetivo entra apenas se não atrasar o gate; e o roadmap é revisto com dados medidos, não com expectativas.

## 59. Barra competitiva

| Dimensão | Handy | VoiceStudio | Objetivo do Parlo |
|---|---|---|---|
| Simplicidade | Forte | Amplo, com gestor de modelos | Automático por omissão |
| Voz própria | Não é o foco | Ferramentas de voz separadas | Caminho principal |
| PT-PT | Transcrição | Cobertura variável | Variante explícita, sem promessas |
| Seleção de modelos | Limitada | Manual | Auto Select com fallback |
| Licenças | Do projeto | Notas por engine | Manifesto por artefacto e gate automático |
| Privacidade | Local | Local com downloads | Local por omissão e auditoria de rede |
| Extensibilidade | Moderada | Alta | Alta, com contratos versionados |

# PARTE XI · OPEN SOURCE, DISTRIBUIÇÃO E GOVERNANÇA

## 60. Repositório e ficheiros obrigatórios

| Ficheiro | Estado | Função |
|---|---|---|
| `README.md` | Existe | Produto, instalação, estado real, secção de licenciamento |
| `LICENSE` | Existe | Apache-2.0 do código original |
| `NOTICE` | Existe | Aviso de atribuição exigido pela Apache-2.0 |
| `LICENSES.md` | Existe, gerado | Índice por módulo, licença de código, pesos e vozes |
| `THIRD_PARTY_NOTICES.md` | Existe, gerado | Avisos de terceiros |
| `licenses/` | Existe | Textos integrais preservados, incluindo MIT histórico |
| `CONTRIBUTING.md` | Existe | Regras de contribuição e proveniência |
| `CODE_OF_CONDUCT.md` | Existe | Conduta |
| `SECURITY.md` | Existe | Relato de vulnerabilidades |
| `SUPPORT.md` | Existe | Onde pedir ajuda |
| `GOVERNANCE.md` | Existe | Como decisões são tomadas |
| `ACCEPTABLE_USE.md` | Existe | Uso aceitável, incluindo voz |
| `docs/PRIVACY.md` | Existe, a reforçar | Dados, retenção e eliminação |
| `docs/MODEL_LICENSES.md` | Existe | Licenças por artefacto de modelo |
| `catalog/models.json` | Existe | Catálogo com hash, licença e estado |
| Documento de voz | Em falta | Política de voz e consentimento, pública |
| Política de modelos externos | Em falta | Como instalar modelos fora do catálogo |

O README declara o estado real, diz o que não funciona e liga para `LICENSES.md`. Nenhum badge afirma conformidade, suporte ou desempenho que não exista.

## 61. Licença do código, DCO e contribuições

O código original está sob Apache-2.0 desde que a titularidade do histórico foi confirmada. As releases até v0.1.7 mantêm o texto MIT com que foram publicadas, preservado em `licenses/`. Não há relicenciamento retroativo.

| Regra | Decisão |
|---|---|
| Assinatura de commits | DCO, verificado no CI |
| Transferência de direitos | Não exigida |
| Proveniência | Contribuições de terceiros identificadas e licenciadas de forma compatível |
| Código sem proveniência | Recusado |
| Contribuições empresariais | Aceites nas mesmas condições |
| Alteração futura de licença | Só com acordo dos titulares e registo público |

## 62. Releases, artefactos e avisos

Cada arquivo de release inclui `LICENSE`, `NOTICE`, `THIRD_PARTY_NOTICES.md`, SBOM CycloneDX, o índice de licenças, hashes dos artefactos e a lista de componentes com versões. Não inclui pesos de modelos, vozes, runtimes GPL nem amostras de áudio.

| Regra | Efeito |
|---|---|
| Release só com o gate verde | Nenhuma publicação com licença pendente |
| Notas de release | O que a versão faz, o que não faz, limitações e números medidos |
| Uma release por lote de trabalho | Sem publicação em série para o mesmo conjunto de alterações |
| Verificação de atualização na aplicação | Lê a última release estável e mostra as notas antes de instalar |
| Release antiga com problema | Não é reescrita nem apagada; é corrigida por documentação e, quando aplicável, pela disponibilização do código correspondente |

## 63. Transparência na aplicação

Uma área chamada «Licenças e código aberto» apresenta, por componente: nome, versão, licença, ligação para a licença original, atribuição exigida e se é obrigatório, opcional ou externo. Antes de instalar um componente com licença própria, aparece uma explicação curta do que essa licença implica, em linguagem simples. A área inclui também os modelos instalados, com a licença de cada um, e o que foi apagado quando o utilizador remove um modelo.

Regra de equilíbrio: a informação está a dois cliques e nunca interrompe o fluxo normal. Avisos jurídicos longos ficam na documentação; a aplicação diz o essencial numa frase.

## 64. Governação

Decisões de arquitetura e de licenciamento são registadas com data, contexto e consequências. Alterações que afetem licenças, privacidade ou direitos de voz exigem revisão explícita e não entram por rotina. O projeto mantém um registo de decisões abertas na secção 68 e um registo de alterações por versão do plano.

# PARTE XII · DECISÕES E RISCOS

## 65. Decisões fechadas

| # | Decisão | Consequência |
|---|---|---|
| 1 | Voz própria é o caminho principal | Clonagem antes de voz neutra; fallback explícito |
| 2 | Nenhum modelo é default sem gate interno | Fim das promoções por leitura de model card |
| 3 | Argos fora do núcleo enquanto houver AGPL | Tradução por NMT aprovado por par |
| 4 | Piper opcional e externo | Núcleo sem GPL; vozes com licença própria |
| 5 | Pesos e vozes nunca nos arquivos de release | Catálogo com hash e licença por artefacto |
| 6 | Licença Apache-2.0 no código original | `LICENSE` e `NOTICE`; MIT histórico preservado |
| 7 | DCO sem transferência de direitos | Contribuições verificadas no CI |
| 8 | Carbono local-first | Nenhuma telemetria; atualização é a única chamada |
| 9 | Model Hub decide, utilizador não compara | Interface com Automático e override avançado |
| 10 | Nenhuma capacidade anunciada antes de medida | README e notas de release limitados ao real |
| 11 | A orquestração é determinística, sem modelo de aprendizado a escolher modelos | Regras, manifestos e resultados medidos explicam cada escolha |
| 12 | Nenhum benchmark pesado corre na máquina do utilizador | Nível A no arranque, nível B na primeira utilização, nível C em CI |

## 66. Decisões que o Parlo toma automaticamente

- Qual o motor de reconhecimento, tradução e voz para a tarefa, idioma, variante e hardware.
- Quando degradar para voz neutra ou para texto, e como o dizer.
- Que modelo descarregar quando o utilizador escolhe uma capacidade e não um modelo.
- Como dividir unidades de tradução e quando emitir voz.
- Que perfil de voz é compatível com que motor e que representação derivar.
- Quando recusar um artefacto por licença, hash ou incompatibilidade.
- Quando mudar de motor, entre segmentos, respeitando permanência mínima e histerese.
- Quando invalidar um resultado de cache e repetir a validação técnica.
- Quando promover um modelo de compatível a recomendado, depois de validação no dispositivo.

## 67. Decisões pendentes de teste interno

| # | Pendente | Como fecha |
|---|---|---|
| 1 | Checkpoint EN→PT que traduza bem | Conjunto de 100 frases, licença confirmada, limiar de fidelidade |
| 2 | Clonador para PT-PT em CPU | Identidade, inteligibilidade e latência medidas |
| 3 | Melhor clonador em GPU | Comparação cega com o mesmo perfil e corpus |
| 4 | Motor de reconhecimento por perfil de hardware | Benchmark nos perfis da secção 47 |
| 5 | Diarização utilizável em reuniões | Taxa de erro de segmentação em gravações reais |
| 6 | Viabilidade de marcador de conteúdo sintético | Prova técnica por formato de exportação |
| 7 | Custo de memória de clonagem em 16 GB | Medição de RSS e VRAM por motor |

## 68. Questões que exigem decisão humana ou validação jurídica

| # | Questão | Quem decide |
|---|---|---|
| 1 | Qualificação de obra combinada ao distribuir um adaptador GPL ao lado do núcleo | Parecer jurídico em Portugal e UE |
| 2 | Se o adaptador Piper pode ser oferecido no instalador como opção | Parecer, com factos e versões |
| 3 | Forma de divulgação de conteúdo sintético que satisfaça o Artigo 50 | Parecer, com o texto do regulamento aplicável |
| 4 | Tratamento de embeddings de voz como dado biométrico em cada finalidade | Parecer, com base na finalidade concreta |
| 5 | Dever de informar participantes de reuniões gravadas em cada jurisdição | Utilizador e parecer |
| 6 | Texto final da política de voz e do uso aceitável | Autor do projeto |

## 69. Riscos por severidade

| Nível | Risco | Mitigação |
|---|---|---|
| Crítico | Clonagem de voz sem consentimento | Registo obrigatório, eliminação total, uso aceitável, recusa de conteúdo enganoso |
| Crítico | Tradução EN→PT sem modelo | Nenhum par declarado até gate; pivô assinalado |
| Alto | Dependência AGPL no núcleo | Saída do Argos e gate por âmbito |
| Alto | Promessa de identidade vocal que o motor não cumpre | Medição por motor e aviso explícito |
| Alto | Distribuição de pesos sem licença | Catálogo com estado de verificação e bloqueio de download |
| Médio | Complexidade de Model Hub atrasar o produto | Fases 6 a 8 em sequência, sem antecipar interface |
| Médio | Orquestrador a oscilar entre motores | Permanência mínima, histerese, circuit breaker e limite de tentativas |
| Médio | Benchmark a competir com o trabalho do utilizador | Nível B limitado a segundos e interrompido sob pressão de memória |
| Médio | Cache a reutilizar resultados de outro ambiente | Chave com modelo, hash, runtime, quantização, hardware e versão do benchmark |
| Baixo | Manifesto desatualizado a esconder uma limitação | Versão de manifesto no catálogo e validação no CI |
| Médio | Desempenho em hardware modesto | Perfis medidos e degradação explícita |
| Médio | Deriva entre README, catálogo e notas de release | Gate de coerência antes de publicar |
| Baixo | Duplicação de documentação de licenças | Índice gerado a partir de uma fonte única |

## 70. Checklist final antes da primeira release

- Tradução PT↔EN com par aprovado e licença fechada; nenhuma dependência AGPL no núcleo.
- Voice Profile com consentimento, pré-visualização e eliminação verificada.
- Pelo menos um clonador medido em PT-PT, com identidade e latência publicadas.
- Model Hub a instalar, verificar, trocar e remover, com licença visível.
- Auto Select e fallback aplicados com aviso e registo.
- Benchmarks reproduzíveis, com hardware, versões, corpus e data.
- `LICENSE`, `NOTICE`, avisos, SBOM, índice e hashes nos artefactos.
- Área «Licenças e código aberto» na aplicação.
- `docs/PRIVACY.md` e política de voz publicados e coerentes com o código.
- Auditoria de rede sem ligações externas durante a sessão.
- Acessibilidade verificada nos fluxos principais.
- Instalação limpa, atualização e rollback testados.
- Notas de release que digam o que não funciona.
- Orquestrador a passar os critérios de aceitação da secção 42, com testes sem modelos reais e um modelo real validado no dispositivo.
- Manifesto de cada modelo candidato com evidência declarada, validada e limitações conhecidas.
