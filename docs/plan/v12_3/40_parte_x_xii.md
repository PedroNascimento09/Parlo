# PARTE XI · QUALIDADE, ROADMAP E FASES

## 67. Lições de Handy, VoiceStudio e Wispr Flow

| Projeto | O que faz bem | O que o Parlo aprende | O que não copia |
|---|---|---|---|
| Handy | Fluxo simples, atalho, VAD, escolha entre Whisper e Parakeet, inserção no campo ativo | Clareza de fluxo e custo baixo de instalação | A ausência de gestão de modelos e de voz |
| VoiceStudio | Gestor de modelos extensível, deteção de dispositivo, instalação sob confirmação | Catálogo amplo e instalação explícita | A exposição da complexidade ao utilizador |
| Wispr Flow | Ditado com tradução e experiência polida | Barra flutuante discreta e inserção fiável | Dependência de serviço externo |

Nenhuma destas comparações afirma superioridade. O Parlo pode ser melhor na combinação de voz própria, execução local e governação de licenças; e pode ser pior em maturidade, suporte de plataformas e polimento durante muito tempo.

## 68. Roadmap incremental

Cada release é instalável, útil por si só e medida. Os números de versão são sequenciais a partir da última tag publicada e não correspondem a marcos internos.

| Release | Nome | O que o utilizador passa a poder fazer | Gate de saída |
|---|---|---|---|
| Próxima | Tradução fechada | Traduzir PT↔EN sem dependência AGPL, com par aprovado | G-MT: nenhum par declarado sem corpus representativo |
| Seguinte | Voz própria | Criar Voice Profile e ouvir uma tradução na própria voz | G-Voice: consentimento, qualidade e identidade medidos |
| Seguinte | Modelos | Instalar, trocar e remover modelos com licença e hash visíveis | G-Models |
| Seguinte | Automático | Auto Select e fallback a funcionar em todas as tarefas | G-Select |
| Seguinte | Reuniões Lite | Importar gravação e obter transcrição com falantes | G-Meeting-Lite |
| Depois | Live | Conversa bidirecional traduzida na voz de cada participante, dentro das aplicações de comunicação que o utilizador já usa | G-Live |
| Depois | Plataforma | API estável, SDK e plugins | G-Platform |

O Live é o objetivo central do produto e, por isso, não cabe numa única release. Entra em três passos com gates separados: legendas traduzidas durante a chamada; voz traduzida num sentido, na voz autorizada de quem falou; e conversa bidirecional entre duas pessoas, cada uma a ouvir a voz da outra na sua língua. O primeiro passo não depende de clonagem e por isso não fica preso a ela; os outros dois dependem das fases 3, 4 e 8.

G-Live exige: duas pessoas em chamadas reais, uma delas em português europeu; identidade vocal medida e publicada por par; encaminhamento de áudio verificado na aplicação usada, com a combinação de aplicação, sistema operativo e dispositivo virtual registada; P50 e P95 por sentido; sessão de 30 minutos sem degradação; e recusa explícita de troca silenciosa de voz.

Regra: as releases só avançam quando os gates fecham. Não há calendário neste documento, e uma release pode demorar o tempo que o gate exigir.

O consentimento e a transparência não são uma fase final. Entram com a primeira capacidade de voz: um perfil só existe com registo de consentimento, e uma exportação só sai com identificação de conteúdo sintético. Antes disso, a funcionalidade de voz não é publicada, mesmo que o motor já sintetize.

## 69. Fases de execução e critérios de conclusão

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
| 11 | Live com legendas | Participantes leem a tradução na língua de destino durante a chamada, sem síntese | Duas pessoas em chamada real, latência de legenda com P95 medido e falante atribuído |
| 12 | Live com voz num sentido | Um participante ouve a tradução na voz autorizada do outro | Identidade medida no par, encaminhamento verificado na aplicação usada e registado, sem troca silenciosa de voz |
| 13 | Live bidirecional | Dois participantes conversam, cada um na sua língua e com a voz do outro | Sessão de 30 minutos sem degradação, interrupções tratadas, P50 e P95 publicados por sentido e por hardware |

As fases 1 a 3 têm precedência. A clonagem antes da voz neutra é ordem de produto, não preferência: sem perfil de voz e sem tradução fiável, a clonagem amplifica um erro em vez de criar valor.

Acresce uma fase própria, anterior à integração de clonadores: fluxo de consentimento completo, registo por perfil, avisos contextuais, etiqueta de síntese na interface, metadados de exportação e revogação com eliminação verificada. Critério de conclusão: criar, ouvir, exportar, revogar e apagar um perfil, com o ficheiro exportado a identificar a origem sintética e sem qualquer escrita fora da pasta de dados.

## 70. Definição de concluída e regras de corte

Uma release está concluída quando: instala em máquina limpa e atualiza sobre a anterior; o rollback funciona; o benchmark do corpus está atualizado com hardware e versões; a matriz de licenças foi revista para tudo o que a release inclui; a auditoria de rede mostra zero ligações externas além da consulta ao catálogo; o changelog declara limitações conhecidas; a acessibilidade mínima foi verificada nos fluxos principais; cada funcionalidade está classificada como experimental, preview, beta ou estável; e o instalador está dentro do orçamento de tamanho ou o desvio está explicado.

Validação com utilizadores externos é um gate próprio, não uma condição de todo o desenvolvimento. Enquanto não existirem testadores reais, os testes internos, funcionais e de acessibilidade podem avançar, e o produto nunca anuncia resultados de usabilidade que não foram medidos. O gate de usabilidade fica registado como não realizado, não como aprovado.

Regras de corte: se o gate falha, a release não sai e a anterior continua a ser a estável; se um modelo não passa, a release sai sem ele com a alternativa já aprovada; trabalho fora do objetivo entra apenas se não atrasar o gate; e o roadmap é revisto com dados medidos, não com expectativas.

## 71. Barra competitiva

| Dimensão | Handy | VoiceStudio | Objetivo do Parlo |
|---|---|---|---|
| Simplicidade | Forte | Amplo, com gestor de modelos | Automático por omissão |
| Voz própria | Não é o foco | Ferramentas de voz separadas | Caminho principal |
| PT-PT | Transcrição | Cobertura variável | Variante explícita, sem promessas |
| Seleção de modelos | Limitada | Manual | Auto Select com fallback |
| Licenças | Do projeto | Notas por engine | Manifesto por artefacto e gate automático |
| Privacidade | Local | Local com downloads | Local por omissão e auditoria de rede |
| Extensibilidade | Moderada | Alta | Alta, com contratos versionados |

# PARTE XII · OPEN SOURCE, DISTRIBUIÇÃO E GOVERNANÇA

## 72. Repositório e ficheiros obrigatórios

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
| `docs/COMPLIANCE.md` | Existe | Checklist interna: licenças, voz, privacidade, Artigo 50, distribuição e revisão por release |

O README declara o estado real, diz o que não funciona e liga para `LICENSES.md`. Nenhum badge afirma conformidade, suporte ou desempenho que não exista.

## 73. Licença do código, DCO e contribuições

O código original está sob Apache-2.0 desde que a titularidade do histórico foi confirmada. As releases até v0.1.7 mantêm o texto MIT com que foram publicadas, preservado em `licenses/`. Não há relicenciamento retroativo.

| Regra | Decisão |
|---|---|
| Assinatura de commits | DCO, verificado no CI |
| Transferência de direitos | Não exigida |
| Proveniência | Contribuições de terceiros identificadas e licenciadas de forma compatível |
| Código sem proveniência | Recusado |
| Contribuições empresariais | Aceites nas mesmas condições |
| Alteração futura de licença | Só com acordo dos titulares e registo público |

## 74. Releases, artefactos e avisos

Cada arquivo de release inclui `LICENSE`, `NOTICE`, `THIRD_PARTY_NOTICES.md`, SBOM CycloneDX, o índice de licenças, hashes dos artefactos e a lista de componentes com versões. Não inclui pesos de modelos, vozes, runtimes GPL nem amostras de áudio.

| Regra | Efeito |
|---|---|
| Release só com o gate verde | Nenhuma publicação com licença pendente |
| Notas de release | O que a versão faz, o que não faz, limitações e números medidos |
| Uma release por lote de trabalho | Sem publicação em série para o mesmo conjunto de alterações |
| Verificação de atualização na aplicação | Lê a última release estável e mostra as notas antes de instalar |
| Release antiga com problema | Não é reescrita nem apagada; é corrigida por documentação e, quando aplicável, pela disponibilização do código correspondente |

## 75. Transparência na aplicação

Uma área chamada «Licenças e código aberto» apresenta, por componente: nome, versão, licença, ligação para a licença original, atribuição exigida e se é obrigatório, opcional ou externo. Antes de instalar um componente com licença própria, aparece uma explicação curta do que essa licença implica, em linguagem simples. A área inclui também os modelos instalados, com a licença de cada um, e o que foi apagado quando o utilizador remove um modelo.

Regra de equilíbrio: a informação está a dois cliques e nunca interrompe o fluxo normal. Avisos jurídicos longos ficam na documentação; a aplicação diz o essencial numa frase.

Além das licenças, a área de transparência mostra os perfis de voz existentes, a origem declarada de cada um, a finalidade autorizada, a data do consentimento e a versão do aviso aceite. A revogação e a eliminação estão no mesmo lugar, porque um direito que exige procurar noutro ecrã não é um direito.

Antes de exportar áudio sintetizado, a aplicação mostra o que vai no ficheiro: marca de síntese, metadados de proveniência e um lembrete de que a divulgação pode ser exigida a quem publica.

## 76. Governação

Decisões de arquitetura e de licenciamento são registadas com data, contexto e consequências. Alterações que afetem licenças, privacidade ou direitos de voz exigem revisão explícita e não entram por rotina. O projeto mantém um registo de decisões abertas na secção 80 e um registo de alterações por versão do plano.

# PARTE XIII · DECISÕES E RISCOS

## 77. Decisões fechadas

| # | Decisão | Consequência |
|---|---|---|
| 1 | Voz própria é o caminho principal | Clonagem antes de voz neutra; fallback explícito |
| 2 | Nenhum modelo é default sem gate interno | Fim das promoções por leitura de model card |
| 3 | Argos fora do núcleo e das dependências obrigatórias | Tradução por NMT aprovado por par |
| 4 | Piper opcional e externo | Núcleo sem GPL; vozes com licença própria |
| 5 | Pesos e vozes nunca nos arquivos de release | Catálogo com hash e licença por artefacto |
| 6 | Licença Apache-2.0 no código original | `LICENSE` e `NOTICE`; MIT histórico preservado |
| 7 | DCO sem transferência de direitos | Contribuições verificadas no CI |
| 8 | Local-first sem telemetria | A consulta ao catálogo é a única chamada externa |
| 9 | Model Hub decide, utilizador não compara | Interface com Automático e override avançado |
| 10 | Nenhuma capacidade anunciada antes de medida | README e notas limitados ao real |
| 11 | Orquestração determinística, sem meta-modelo | Regras, manifestos e medições explicam cada escolha |
| 12 | Sem benchmarks pesados na máquina do utilizador | Níveis A, B e C separados |
| 13 | Aplicação sem pesos, com orçamento próprio | Objetivo de 100 MB, limite de referência de 150 MB |
| 14 | Biblioteca independente do instalador, catálogo no repositório principal | Pesos fora do Git; separação do catálogo só com justificação técnica |
| 15 | Manifestos assinados com Ed25519 | Chave pública na aplicação; privada fora do repositório |
| 16 | Sem Authenticode nesta fase | Hashes e instruções, sem os apresentar como assinatura |
| 17 | Clonagem da própria voz e de terceiros autorizados | Declaração obrigatória; engano proibido |
| 18 | Corpus de avaliação original sob CC0-1.0 | Texto próprio, sem dados privados de terceiros |
| 19 | Latência publicada em P50 e P95 por perfil | 2,5 s é objetivo do perfil equilibrado, não garantia |
| 20 | Sem parecer jurídico nesta fase | Checklist interna de conformidade revista por release |

## 78. Decisões que o Parlo toma automaticamente

- Qual o motor de reconhecimento, tradução e voz para a tarefa, idioma, variante e hardware.
- Quando degradar para voz neutra ou para texto, e como o dizer.
- Que modelo descarregar quando o utilizador escolhe uma capacidade e não um modelo.
- Como dividir unidades de tradução e quando emitir voz.
- Que perfil de voz é compatível com que motor e que representação derivar.
- Quando recusar um artefacto por licença, hash ou incompatibilidade.
- Quando mudar de motor, entre segmentos, respeitando permanência mínima e histerese.
- Quando invalidar um resultado de cache e repetir a validação técnica.
- Quando promover um modelo de compatível a recomendado, depois de validação no dispositivo.

- Quando repetir um aviso de voz sintética e quando já existe consentimento válido para a mesma finalidade.
- Que motor usar quando o preferido não suporta a marcação exigida, com a limitação declarada.
- Que informação de proveniência acompanha cada formato de exportação.

## 79. Decisões pendentes de teste interno

| # | Pendente | Como fecha |
|---|---|---|
| 1 | Checkpoint EN→PT que traduza bem | Conjunto de 100 frases, licença confirmada, limiar de fidelidade |
| 2 | Clonador para PT-PT em CPU | Identidade, inteligibilidade e latência medidas |
| 3 | Melhor clonador em GPU | Comparação cega com o mesmo perfil e corpus |
| 4 | Motor de reconhecimento por perfil de hardware | Benchmark nos perfis da secção 59 |
| 5 | Diarização utilizável em reuniões | Taxa de erro de segmentação em gravações reais |
| 6 | Viabilidade de marcador de conteúdo sintético | Prova técnica por formato de exportação |
| 7 | Custo de memória de clonagem em 16 GB | Medição de RSS e VRAM por motor |
| 8 | Marca de água viável em cada motor candidato | Verificar suporte real e formato, sem presumir |
| 9 | Metadados de proveniência por formato de exportação | Prova técnica em WAV, FLAC e MP4 |
| 10 | Deteção de voz sintética como aviso ao utilizador | Avaliar utilidade e falsos positivos antes de prometer |

## 80. Questões que exigem decisão humana ou validação jurídica

O Parlo é um projeto pessoal, gratuito, open-source e público, sem venda, subscrição, publicidade ou rendimento associado. Nesta fase não há parecer jurídico contratado e a ausência dele não bloqueia desenvolvimento nem publicação. Também não é apresentado como conformidade jurídica: o projeto cumpre o que consegue verificar e identifica o que exige interpretação.

| Obrigação ou questão | Estado no projeto | O que falta |
|---|---|---|
| Licenças de código, dependências e pesos | Verificadas por artefacto, com gate automático | Manter atualizado a cada dependência nova |
| Avisos e atribuições | `NOTICE`, avisos de terceiros e índice gerados | Revisão manual antes de cada release |
| Consentimento de voz | Registo local com finalidade, âmbito e data | Texto final da política de voz |
| Dados vocais e retenção | Local, apagável, sem envio | Confirmar cada finalidade nova |
| Regulamento Europeu da IA, Artigo 50 | Marcação e divulgação identificadas por interveniente | Forma técnica de marcação por formato |
| RGPD | Minimização, finalidade e eliminação implementadas | Qualificação de embeddings por finalidade |

Três pontos ficam explicitamente como interpretação jurídica, sem bloquear o trabalho: a qualificação de obra combinada ao distribuir um adaptador GPL ao lado do núcleo; a forma de divulgação de conteúdo sintético que satisfaça o Artigo 50; e o tratamento de representações vocais como dado biométrico em cada finalidade concreta.

Consulta jurídica só se justifica quando existir dúvida material e concreta que não se resolva com documentação oficial, licença aplicável ou análise técnica razoável — por exemplo, uma distribuição que combine binários copyleft com o núcleo, ou um pedido de utilização comercial que dependa dessa qualificação. O acompanhamento fica numa checklist interna de conformidade, revista a cada release.

A forma exata de divulgação de conteúdo sintético em cada meio continua a ser interpretação, e o Parlo resolve-a de forma conservadora: etiqueta na interface, metadados no ficheiro e aviso antes da partilha. Se o Artigo 50 exigir mais num caso concreto, a limitação é declarada em vez de improvisada.

## 81. Riscos por severidade

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
| Alto | Aplicação a crescer com o catálogo | Pesos fora do instalador; regra de dimensionamento na secção 44 |
| Médio | Modelo atualizado a quebrar o que funcionava | Instalação ao lado, validação antes da troca e rollback |
| Médio | Dependências partilhadas apagadas com um modelo | Remoção protege referências de outros modelos |
| Médio | Importação externa a executar código | Nenhum código do pacote corre; importados fora do automático |
| Médio | Desempenho em hardware modesto | Perfis medidos e degradação explícita |
| Médio | Deriva entre README, catálogo e notas de release | Gate de coerência antes de publicar |
| Baixo | Duplicação de documentação de licenças | Índice gerado a partir de uma fonte única |

## 82. Checklist final antes da primeira release

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
- Instalador sem pesos; biblioteca a crescer sem alterar o tamanho da instalação.
- Catálogo atualizável sem reinstalar a aplicação, com descoberta que não descarrega.
- Instalar, substituir e remover modelos individualmente, sem tocar em perfis de voz nem em preferências.

- Consentimento obrigatório antes de qualquer perfil, com origem declarada e versão do aviso registada.
- Avisos contextuais nos sete momentos definidos e sem repetição desnecessária.
- Etiqueta de voz sintética na interface e metadados de proveniência no ficheiro exportado.
- Revogação e eliminação verificadas, incluindo caches e representações derivadas.
- Manifesto de cada clonador com capacidades de transparência declaradas.
- Nenhuma afirmação de marca de água ou de identificação sem verificação no motor usado.
- Remoção de um modelo a proteger dependências partilhadas e a oferecer alternativa quando está em uso.
