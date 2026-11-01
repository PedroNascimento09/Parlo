# PARTE V · LICENÇAS E DIREITOS

## 21. Licenciamento por artefacto

O Parlo distribui tipos de artefacto diferentes, e cada tipo tem a sua licença. A regra que organiza todo o resto: **nada herda a licença de outra coisa**. O código Apache-2.0 do motor não licencia um checkpoint, uma voz, um dataset ou um pacote de terceiros.

| Artefacto | Licença no projeto | Obrigações | Onde aparece |
|---|---|---|---|
| Código original (motor, CLI, app, ferramentas, SDK) | Apache-2.0 | `LICENSE`, `NOTICE`, aviso de alterações | Repositório e arquivos de release |
| Releases até v0.1.7 | MIT, preservada | Texto original intacto | `licenses/MIT-Parlo-0.1.x.txt` |
| Adaptadores de modelos | Licença própria conforme o código e as dependências | Documentada por adaptador | Repositório e `LICENSES.md` |
| Adaptador Piper | GPL-3.0-or-later, porque usa runtime GPL | Texto e código correspondente disponíveis | `engines/tts-piper`, fora dos arquivos de release |
| Dependências Rust e Python | As dos respetivos autores | Avisos agregados, SBOM | `THIRD_PARTY_NOTICES.md` e SBOM CycloneDX |
| Pesos de modelos | Por artefacto, registada no catálogo | Atribuição e condições por checkpoint | `catalog/models.json` e `docs/MODEL_LICENSES.md` |
| Vozes | Por voz, registada individualmente | Nome do autor, origem, condições | Catálogo e área de licenças na aplicação |
| Datasets de avaliação | Por dataset | Uso interno, sem redistribuição por omissão | Documentação de benchmarks |

Consequências práticas: o instalador não transporta pesos; o README declara a licença do código e remete para `LICENSES.md` para tudo o resto; e nenhuma página do projeto descreve uma licença de módulo como se cobrisse os seus componentes.

## 22. Dependências e risco copyleft

Separar processos é uma decisão de engenharia; a qualificação de obra combinada é uma questão jurídica que este documento não decide. O que o projeto faz é reduzir a exposição e documentar a fronteira com precisão.

| Componente | Licença | Risco | Decisão |
|---|---|---|---|
| piper1-gpl | GPL-3.0-or-later | Alto se ligado ao núcleo | Fora do programa do motor; adaptador externo opcional |
| espeak-ng | GPL-3.0 | Alto: é a razão da licença do Piper | Não é dependência do núcleo |
| argostranslate 1.11.0 | MIT declarado | Médio | Em revisão |
| MiniSBD | AGPLv3 | Alto: obrigações de rede | Presente por dependência transitiva do Argos; motivo da saída do Argos do núcleo |
| CTranslate2 | MIT | Baixo | Aceite |
| sherpa-onnx | Apache-2.0 | Baixo | Aceite |
| faster-whisper | MIT | Baixo | Aceite |
| numpy, psutil, truststore | BSD, BSD, MIT | Baixo | Aceite |
| pyannote (pesos) | Condições de acesso | Médio | Não redistribuir pesos; download pelo utilizador |

Regras de fronteira: o núcleo não importa código GPL nem AGPL; os adaptadores copyleft vivem em processos separados, com a sua licença, e são instalados por ação explícita do utilizador; o produto funciona sem eles; o protocolo entre núcleo e adaptador é público e documentado; e nenhuma release agrega um binário copyleft ao pacote principal. A documentação diz que esta fronteira é técnica e que a qualificação jurídica de distribuição conjunta fica para parecer, quando existir.

Decisão fechada: o Argos deixa de ser dependência obrigatória do núcleo. A integração pode permanecer como adaptador externo opcional, documentado, mas nenhuma cadeia que inclua MiniSBD volta ao caminho principal sem análise específica e decisão registada.

## 23. Pesos, vozes e datasets

Cada artefacto descarregável tem um registo com: identificador e versão, origem oficial, URL, hash, licença, atribuição obrigatória, condições conhecidas e estado de verificação. O procedimento é o mesmo para todos:

- Abrir a fonte oficial e ler a licença declarada para aquele checkpoint, não para a família.
- Registar o commit ou a revisão exata descarregada.
- Calcular o hash do ficheiro distribuído e guardá-lo no catálogo.
- Registar a licença das vozes ou dos dados em campo próprio, separado do código.
- Marcar como não verificado tudo o que não foi confirmado, e impedir a admissão pelo orquestrador enquanto assim estiver.

Estado atual: as três vozes Piper incluídas estão verificadas como MIT no repositório de vozes, o checkpoint Qwen3-ASR 0.6B está verificado como Apache-2.0, e o pacote Argos é o único artefacto ainda marcado como não verificado — precisamente o que está a sair. Nenhum peso de modelo é distribuído dentro de um arquivo de release.

## 24. Modelos bloqueados e catálogo experimental

| Categoria | Comportamento |
|---|---|
| Aprovado | Download em um clique, hash verificado, licença visível |
| Experimental | Área separada, aviso com fonte, licença, desconhecidos e consequência, opt-in explícito |
| Bloqueado | Não aparece para download; aparece a razão na documentação |
| Externo | Instalado fora do Parlo, com contrato de processo e licença própria |

Ficam bloqueados: F5-TTS (pesos CC-BY-NC), XTTS-v2 (licença não comercial), NLLB-200 e SeamlessM4T v2 (CC-BY-NC), e qualquer checkpoint cuja licença não tenha sido confirmada. Ficam experimentais: Qwen3-ASR, VoxCPM, VoiceDesign e diarização com pesos condicionados. Ficam externos: Piper e Argos, se o Argos voltar.

Um modelo bloqueado pode ser usado por quem o instalar por sua conta, mas o Parlo não o descarrega, o PIMO não o admite na seleção automática e o produto não o apresenta como capacidade.

Estado do Argos: sai do núcleo e das dependências obrigatórias. Fica documentado como possibilidade experimental futura, sujeita à resolução da cadeia AGPL, e não aparece na distribuição normal.

## 25. Direitos de voz e uso aceitável

O Parlo exige o mínimo necessário para ser um projeto responsável, sem transformar a utilização normal num formulário. A regra de fundo: clonagem autorizada para tradução, acessibilidade e criação própria é o caso normal; utilização enganosa não é.

| Utilização | Estado |
|---|---|
| Falar na própria voz, em qualquer língua | Permitida |
| Voz de outra pessoa com autorização declarada | Permitida, com registo |
| Tradução e acessibilidade com voz autorizada | Permitida |
| Dobragem de conteúdo próprio ou licenciado | Permitida |
| Clonagem sem autorização | Proibida |
| Fraude, burla ou vantagem obtida por impostura | Proibida |
| Impersonação enganosa de pessoa ou organização | Proibida |
| Assédio, intimidação ou humilhação com a voz de alguém | Proibida |
| Declarações falsas atribuídas a terceiros | Proibida |

| Situação | Exigência |
|---|---|
| A própria voz | Declaração de que a voz é do utilizador |
| Voz de outra pessoa | Declaração de autorização, com finalidade e âmbito, registada localmente |
| Voz de ator, dataset ou voz comercial | Autorização e licença compatíveis; nunca como exemplo no repositório |
| Vozes de exemplo na documentação | Só sintéticas neutras ou do próprio autor do exemplo |
| Vozes pré-instaladas | Não existem |
| Cópia do repositório | Não contém amostras privadas, perfis ou caches |

A declaração é uma afirmação do utilizador, não uma prova de autorização, e o produto di-lo sem rodeios. O Parlo não verifica identidade, não certifica mandatos e não apresenta o registo local como garantia jurídica. O que faz é não criar perfil sem declaração, permitir revogar e apagar, identificar a saída sintética e recusar funcionalidades que existam apenas para enganar.

O âmbito completo está em `ACCEPTABLE_USE.md` e na política de voz publicada no repositório.

## 26. Conformidade automática em cada release

A conformidade deixa de depender de memória. O que existe hoje: um gate que lê a cadeia Rust por `cargo metadata`, a cadeia Python pelas dependências fixadas e o catálogo de modelos, e falha quando uma licença falta, está negada, sai da allowlist do âmbito ou aparece numa dependência sem versão fixa; avisos de terceiros e SBOM CycloneDX gerados do mesmo inventário; índice de licenças regenerado e comparado com o commit; e recusa de build enquanto o gate tiver problemas abertos.

| Controle | Efeito | Estado |
|---|---|---|
| Allowlist por âmbito | GPL só em adaptadores opcionais, nunca no núcleo | Implementado |
| Dependências fixadas | Nenhuma versão flutuante em Rust ou Python | Implementado |
| SBOM CycloneDX por release | Inventário com versões e hashes | Implementado |
| `THIRD_PARTY_NOTICES.md` | Avisos agregados | Implementado |
| `LICENSES.md` validado no CI | Índice coerente com o catálogo | Implementado |
| Verificação de artefactos de modelo | Falha quando falta licença, hash ou estado | Implementado no catálogo; falta bloquear a admissão |
| Hashes de binários de release | Registo por artefacto | Implementado |
| Builds reprodutíveis | Sem dependência do ambiente de quem compila | Parcial: falta fixar toolchain e imagem |

> O gate prova coerência de metadados, não conformidade jurídica. Uma dependência nova com licença aprovada na allowlist pode ainda criar obrigações que exigem leitura humana; por isso a allowlist é revista por componente e as exceções ficam registadas na secção 80, não escondidas num ficheiro de configuração.

# PARTE VI · ORQUESTRAÇÃO INTELIGENTE, MODEL HUB E AUTO SELECT

## 27. Arquitetura de orquestração: PIMO e fronteiras

O **Parlo Intelligent Model Orchestrator** é o componente que decide, em cada momento, qual modelo serve a tarefa pedida naquele dispositivo. Não é um modelo de machine learning que aprende a escolher; é um motor de decisão determinístico sobre manifestos, perfil de hardware, licenças e resultados de benchmark guardados localmente. Essa escolha é deliberada: um seletor com regras auditáveis explica-se, testa-se e não surpreende o utilizador.

| Componente | Responsabilidade | Não faz |
|---|---|---|
| PIMO | Filtrar, avaliar, selecionar, compor pipeline, monitorizar e trocar | Não descarrega ficheiros, não guarda perfis de voz |
| Model Hub | Ciclo de vida dos artefactos: download, hash, instalação, atualização, remoção | Não escolhe modelos |
| Auto Select | Regra de produto que expõe o PIMO ao utilizador | Não tem lógica própria de avaliação |
| Hardware Profiler | Mede o dispositivo e o que está livre naquele momento | Não testa modelos |
| Runtime Manager | Arranca, vigia e termina processos de motor | Não decide qual motor |
| Voice Profile Manager | Guarda perfis, representações por motor e consentimento | Não escolhe motor de clonagem |
| Benchmark Store | Guarda resultados com chave de validade | Não interpreta resultados |

### 27.1. Interfaces

| Interface | Entrada | Saída |
|---|---|---|
| `plan(task, locale, mode)` | Tarefa, idioma e variante, modo de utilização, perfil de voz | Plano de execução com candidatos ordenados e razão |
| `admit(model)` | Manifesto e hardware atual | Elegível, eliminado com motivo, ou pendente de teste |
| `probe(model)` | Candidato elegível | Resultado técnico do nível B |
| `observe(metrics)` | Latência, memória, erros da sessão | Evento de degradação ou de estabilidade confirmada |
| `recover(failure)` | Falha tipificada | Próximo candidato ou degradação declarada |
| `explain(plan)` | Plano | Frase legível para a interface e registo técnico |

### 27.2. Regras de fronteira

- O PIMO não escreve no catálogo nem instala nada; pede ao Model Hub e espera confirmação de hash.
- O Model Hub não conhece preferências de qualidade; só estados de artefacto.
- O Voice Profile Manager é a única peça que lê amostras de voz, e o PIMO só recebe metadados de compatibilidade, nunca o áudio.
- Toda a decisão é registada com os sinais que a produziram, para que uma troca de comportamento seja diagnosticável meses depois.

## 28. Manifesto por modelo e níveis de evidência

Cada entrada do catálogo declara, no mínimo: identificador e versão; tarefa; projeto de origem; URL oficial; hash SHA-256; licença do código, dos pesos e de dados ou vozes; idiomas e variantes efetivamente validadas; funcionalidades; arquitetura; runtime; sistemas operativos; CPU, GPU e aceleradores; RAM e VRAM mínimas e recomendadas; armazenamento; formatos e quantizações; compatibilidade com outros componentes; resultados de benchmark; limitações conhecidas; estado de manutenção; proveniência e versão do manifesto.

Capacidades de transparência fazem parte do manifesto de qualquer modelo que sintetize ou clone voz: se suporta metadados de proveniência, se aplica marca de água verificável, qual o formato dessa marca, se permite desativá-la e que informação o ficheiro transporta. Um modelo sem essa declaração não é admitido como clonador recomendado.

### 28.1. Níveis de evidência

| Nível | Significado | Pode ser default? |
|---|---|---|
| Declarado | O fabricante afirma; o Parlo não confirmou nada | Não |
| Documentado | Confirmado em documentação oficial consultada | Não |
| Validado | Passou os testes do Parlo naquele runtime e configuração | Sim |
| Experimental | Funciona, com limitações conhecidas e opt-in | Não |
| Não confirmado | Proveniência, licença ou integridade insuficientes | Não |

Uma capacidade anunciada nunca é apresentada como validada pelo Parlo. O manifesto guarda os dois campos em separado — o que a origem declara e o que o projeto mediu — e a interface mostra sempre o segundo.

### 28.2. Manifesto incompleto

Campo obrigatório em falta impede a entrada no modo automático, mas não esconde o modelo: a entrada aparece como não confirmada, com o campo em falta nomeado. O catálogo atual cobre licença, origem, hash e estado de verificação; faltam cobertura de hardware, latência medida, regra de recomendação e versão de manifesto.

## 29. Descoberta e filtragem eliminatória

Antes de qualquer teste, o PIMO reduz o universo por metadados. Um candidato eliminado nunca é executado.

| Filtro | Critério |
|---|---|
| Tarefa | O manifesto declara a tarefa pedida |
| Idioma e variante | Cobertura declarada e, quando existir, validada para a variante |
| Licença | Compatível com a política do projeto para o âmbito de distribuição |
| Sistema operativo | Suporte confirmado no sistema atual |
| Recursos | RAM, VRAM e disco compatíveis com o que está realmente livre |
| Voz | Capacidade de preservar os requisitos do Voice Profile, quando a clonagem é exigida |
| Integridade | Hash disponível e proveniência verificável |
| Segurança | Sem problemas conhecidos de integridade, execução arbitrária ou vulnerabilidade aberta |

Ordem e custo: filtros de metadados primeiro, hardware depois, licença antes de qualquer download. Um candidato sem licença confirmada para o âmbito ativo é eliminado sem gastar rede nem CPU — uma inversão que poupa tempo e evita distribuir o que não pode ser distribuído.

## 30. Perfil automático do dispositivo

O Hardware Profiler mede o dispositivo localmente e distingue capacidade instalada de recursos realmente disponíveis.

| Medição | Uso |
|---|---|
| Sistema operativo e versão | Compatibilidade de runtime |
| Arquitetura e instruções | Escolha de build e de quantização |
| Núcleos físicos e lógicos | Paralelismo real, não threads teóricas |
| RAM total e livre | Admissão de modelos e margem de segurança |
| GPU e aceleradores | Backend disponível e VRAM livre |
| Backends possíveis | CUDA, ROCm, Metal, DirectML, CPU |
| Disco livre | Download e instalação |
| Velocidade de armazenamento | Tempo de carregamento em modelos grandes |
| Carga atual | Decisão de arranque sem competir com o sistema |

### 30.1. O que nunca é assumido

- Que duas máquinas com a mesma RAM têm o mesmo desempenho.
- Que uma GPU instalada está utilizável pelo backend disponível.
- Que a memória livre no arranque se mantém durante uma sessão.
- Que as especificações teóricas substituem uma medição de latência.

O perfil é obtido sem enviar informação do dispositivo para fora e sem testes agressivos. Nenhuma sonda de stress entra no arranque normal; a validação pesada ocorre em desenvolvimento e em integração contínua.

## 31. Avaliação em três níveis

| Nível | Onde corre | O que produz | Custo |
|---|---|---|---|
| A · estática | Sempre, no arranque | Lista de elegíveis e motivos de eliminação | Desprezável |
| B · validação técnica | Primeira utilização de um candidato naquele dispositivo | Integridade, carregamento, memória, latência, validade | Segundos |
| C · qualidade aprofundada | Desenvolvimento, CI e atualização de modelo | Qualidade, estabilidade, robustez, reprodutibilidade | Minutos a horas |

### 31.1. Nível B

Verifica hash e carregamento, compatibilidade de runtime, memória consumida, tempo de inicialização, latência da primeira e das inferências seguintes, validade do resultado e libertação de recursos no descarregamento. Usa amostras pequenas e representativas. Interrompe quando houver risco de esgotamento de memória, instabilidade ou consumo anómalo. Um teste técnico bem-sucedido não é uma validação de qualidade, e o resultado guarda essa distinção.

### 31.2. Nível C

Avalia qualidade, precisão, naturalidade, inteligibilidade, latência, estabilidade, consumo e reprodutibilidade, com o corpus e as condições registados. Corre no repositório, com os scripts de avaliação existentes — `battery.py`, `stt_bench.py`, `compare.py`, `soak.py` e `stream_bench.py` —, e não na máquina do utilizador. Resultados aprofundados são reutilizados quando modelo, runtime, configuração, ambiente e versão do benchmark coincidem.

## 32. Benchmarks com evidência por tarefa

Uma pontuação única não compara modelos de tarefas diferentes. Cada categoria tem as suas métricas e o seu conjunto de testes.

| Tarefa | Métricas | Cuidados |
|---|---|---|
| Reconhecimento | WER, CER, pontuação e segmentação, timestamps, robustez a ruído, vocabulário | PT-PT e PT-BR em separado; sotaques declarados |
| Tradução | BLEU, chrF, COMET quando disponível, preservação de números e negações, terminologia, fluidez | Ambas as direções; conjunto representativo; validação humana no desenvolvimento |
| Clonagem e síntese | Semelhança de locutor, naturalidade, inteligibilidade, pronúncia, prosódia, consistência entre frases, estabilidade em sessão, latência, memória | Semelhança é indicador, não prova de identidade preservada |
| Tempo real | P50 e P95 do primeiro resultado, do primeiro áudio e do total, atraso acumulado, jitter, perdas | Distinguir latência de componente de latência de experiência |

O corpus de avaliação é escrito para o projeto e versionado com o código: frases curtas e longas, conversa formal e informal, vocabulário técnico, números, datas, nomes próprios e expressões idiomáticas, em português europeu, português brasileiro e inglês, nos dois sentidos de tradução. Os textos originais ficam sob CC0-1.0 e a origem é registada. Não se usam mensagens privadas nem texto de terceiros sem autorização.

Um corpus escrito mede tradução, não reconhecimento. Para reconhecimento são necessárias gravações autorizadas, e a diferença é declarada em cada resultado.

Regra de publicação: um número aparece com modelo, versão, quantização, runtime, threads, hardware, corpus e data. Média sem condições não entra em notas de release. Métricas automáticas não substituem avaliação humana nos pares de línguas e nas vozes que o produto declara suportar.

## 33. Seleção multicritério

O PIMO ordena os candidatos aptos com prioridades fixas e pesos por tarefa, definidos a partir de medições. Nunca inventa pontuações.

| Ordem | Critério | Efeito |
|---|---|---|
| 1 | Requisitos funcionais e legais | Elimina, não pontua |
| 2 | Preservação da identidade vocal, quando pedida | Elimina se não houver suporte técnico |
| 3 | Qualidade mínima validada | Elimina abaixo do limiar da tarefa |
| 4 | Compatibilidade técnica | Elimina se o dispositivo não serve |
| 5 | Estabilidade | Desempata com peso alto |
| 6 | Latência adequada ao modo | Desempata; nunca compensa qualidade em falta |
| 7 | Eficiência de recursos e energia | Desempata no fim |

Incerteza: quando não há dados suficientes para comparar dois candidatos, o PIMO prefere o que já está validado para aquele contexto e regista a decisão como conservadora. Se nenhum candidato cumprir a qualidade mínima, o resultado correto não é escolher o menos mau em silêncio: é declarar que a tarefa não tem motor adequado e degradar segundo a secção 36.

A explicação tem duas camadas: uma frase na interface — «Escolhi este modelo porque cobre PT-PT, cabe em 3,1 GB livres e tem a menor latência medida nesta máquina» — e um registo técnico com os sinais, os pesos e os candidatos eliminados.

Transparência é critério eliminatório, não desempate. Se a política em vigor exigir marcação ou metadados e o candidato não os suportar, o PIMO não o escolhe para essa finalidade e apresenta a alternativa com a limitação explícita. A seleção automática nunca remove um mecanismo de transparência aplicável para ganhar qualidade ou latência.

## 34. Composição de pipeline

O melhor sistema raramente é um só fornecedor. O PIMO escolhe por etapa — reconhecimento, tradução, clonagem, processamento auxiliar — e avalia o conjunto.

| Risco de combinação | Como é tratado |
|---|---|
| Latência acumulada | Orçamento por etapa; soma medida, não estimada |
| Incompatibilidade de formatos | Contrato de amostras, taxas e canais no adaptador |
| Perda de contexto na tradução | Unidades semânticas e contexto de sessão definidos fora dos motores |
| Conflito de memória | Soma de RSS e VRAM dos motores ativos verificada antes de compor |
| Dessincronização de áudio | Timestamps monotónicos no núcleo, não no motor |
| Dependências adicionais | Cada motor extra tem de justificar o seu custo em qualidade medida |

Regra: três modelos individualmente excelentes que produzem pior experiência composta não são escolhidos. A unidade de decisão é o pipeline, não o componente.

## 35. Monitorização de runtime e adaptação

Durante a utilização, o PIMO observa latência real, erros, RAM e VRAM, falhas de inferência, saturação, tempo de resposta e estabilidade. Reage quando as condições mudam, com mecanismos que evitam oscilação.

| Mecanismo | Regra |
|---|---|
| Período mínimo de permanência | Uma configuração fica ativa durante um mínimo antes de poder ser trocada |
| Histerese | A alternativa tem de ser claramente melhor, não marginalmente melhor, para justificar troca |
| Circuit breaker | Falhas repetidas do mesmo motor retiram-no da seleção durante um período definido |
| Limite de tentativas | Número finito de recuperações por sessão, com degradação declarada quando se esgota |
| Limite de recursos | Interrupção segura antes de esgotar memória, com o texto já reconhecido preservado |
| Fronteira segura | A troca acontece entre segmentos e nunca a meio de uma frase ou de uma emissão de voz |

Se a latência subir de forma persistente, o PIMO investiga a causa antes de trocar: pode ser fila acumulada, thermal throttling, contenção de memória ou um modelo inadequado. Trocar de modelo para resolver um problema de fila só piora o diagnóstico.

## 36. Fallback e proteção da identidade vocal

Antes de mudar de motor, o PIMO confirma compatibilidade funcional, linguística, de qualidade mínima, de dados existentes, de perfil de voz e de recursos. Um fallback que perde o requisito essencial da tarefa não é um fallback: é uma degradação, e tem de ser declarada.

| Situação | Comportamento |
|---|---|
| Motor falha, alternativa compatível existe | Troca, sem interromper, com aviso depois do segmento |
| Clonador falha, outro clonador serve o mesmo perfil | Troca apenas se a representação para esse motor existir ou puder ser derivada |
| Clonador falha, nenhum outro serve | Declara que a identidade vocal não pode ser garantida e oferece voz neutra identificada ou só texto |
| Perfil não tem representação para o motor de destino e a amostra foi apagada | Não troca; explica que é necessário gravar de novo |
| Nenhum motor com qualidade mínima | Declara a limitação e mantém o texto |

Regras de voz que o PIMO nunca viola: não assume embeddings compatíveis entre motores; não usa voz genérica como substituição silenciosa; não promete preservação de identidade sem medição naquele motor; e não decide sozinho apagar ou substituir um perfil.

A transparência sobrevive ao fallback. Um motor alternativo que não marque conteúdo sintético não substitui silenciosamente um que marcava: a troca é declarada, a etiqueta na interface mantém-se e o ficheiro exportado diz que a marcação não está disponível naquele motor.

## 37. Model Hub e ciclo de vida

| Estado | Entrada | Saída |
|---|---|---|
| Descoberto | Entra no catálogo | Verificação de proveniência e licença |
| Verificado | Licença, hash e proveniência confirmados | Download disponível |
| Instalado | Ficheiros presentes e hash validado | Validação técnica |
| Validado | Passou o nível B naquele dispositivo | Elegível para seleção automática |
| Ativo | Em uso numa sessão | Desativação ou remoção |
| Desativado | Fora de uso, ficheiros mantidos | Remoção ou reativação |
| Removido | Ficheiros e caches eliminados | Nova instalação exige download |

Regras: nada executa sem hash validado; nenhum download silencioso; espaço verificado antes de descarregar; nenhuma atualização de modelo durante sessão ativa; rollback para a última versão validada sempre disponível; e uma atualização nunca substitui uma versão estável sem validação prévia. Pesos de terceiros nunca entram nos arquivos de release.

## 38. Cache de resultados e reutilização

Cada resultado de validação fica associado a modelo e versão, hash dos ficheiros, runtime e versão, configuração e quantização, hardware relevante, sistema operativo, versão do benchmark, dataset, data e estado de validade.

| Mudança | Efeito na cache |
|---|---|
| Mesma máquina, mesmo tudo | Resultado reutilizado |
| Atualização do modelo ou do runtime | Invalida o resultado do nível B e do nível C |
| Alteração de quantização ou configuração | Invalida |
| Hardware muito diferente | Não reutiliza; repete o nível B |
| Nova versão do benchmark | Invalida o nível C, mantém o histórico para comparação |

O PIMO não repete testes sem razão concreta e não reutiliza resultados de ambientes que não sejam equivalentes. A cache é local, apagável e nunca sai do dispositivo.

## 39. Privacidade e segurança da orquestração

- Nenhum benchmark envia áudio, amostras vocais, embeddings, transcrições ou métricas para terceiros.
- Datasets de avaliação são autorizados e controlados; os de desenvolvimento não são distribuídos com o produto.
- Manifestos, downloads e caches têm verificação de integridade; um manifesto adulterado é rejeitado.
- Nenhum código de modelo corre sem proveniência verificável e hash confirmado.
- Limites de recursos e interrupção segura fazem parte da orquestração, não são opcionais.
- O registo técnico de decisões não inclui conteúdo do utilizador, apenas identificadores de modelo, métricas e motivos.

## 40. Experiência do utilizador e transparência

O utilizador não escolhe motores, não compara modelos, não conhece RAM nem VRAM, não instala dependências e não resolve conflitos. Vê apenas o que lhe é útil: o estado da operação, o modelo em uso quando isso explica alguma coisa, limitações, necessidade de espaço ou de download, e avisos de qualidade ou de preservação de voz.

| Situação | O que aparece |
|---|---|
| Tudo normal | Estado e resultado, sem jargão |
| Escolha relevante | Uma frase com o motivo |
| Troca de motor | Aviso curto, com o que mudou |
| Degradação de voz | Aviso explícito de que já não é a voz da pessoa |
| Falta de espaço ou download | Tamanho, destino e uma ação |
| Nenhum motor adequado | Explicação e a alternativa segura que resta |

O modo automático é a experiência completa, não uma versão reduzida. As definições avançadas existem para quem quer fixar um motor, e nesse caso o produto diz que a escolha deixa de ser automática e passa a ser verificada de outra forma.

## 41. Testes automáticos da orquestração

Testes sem modelos reais, com manifestos e métricas simulados, cobrem o que é lógica pura: filtragem, ordenação, empates, incerteza, histerese, limites de tentativas, circuit breaker, invalidação de cache e composição de pipeline. Testes com modelos reais ficam reservados ao nível B num único modelo pequeno e ao nível C em CI.

| Teste | O que garante |
|---|---|
| Eliminação por licença | Nenhum candidato sem licença sobrevive ao nível A |
| Eliminação por recursos | Um modelo que não cabe nunca é admitido |
| Histerese | Uma variação pequena de latência não muda de motor |
| Circuit breaker | Falhas repetidas retiram o motor e o produto continua |
| Falha de GPU | Cai para CPU com aviso e sem perder texto |
| Perfil sem representação | Não troca de motor e explica porquê |
| Cache inválida | Resultado antigo não é usado depois de mudar a configuração |
| Regressão de qualidade | Uma versão nova com pior qualidade medida não substitui a anterior |

## 42. Critérios de aceitação do PIMO

1. Filtra automaticamente modelos incompatíveis, com motivo registado.
2. Identifica elegíveis sem executar benchmarks extensivos.
3. Valida tecnicamente os candidatos que escolhe, num tempo aceitável.
4. Reutiliza resultados ainda válidos e repete só quando algo mudou.
5. Seleciona modelos diferentes para tarefas diferentes, com justificação.
6. Compõe motores sem degradar o pipeline face a um só motor.
7. Monitoriza desempenho durante a utilização.
8. Deteta e recupera de falhas sem intervenção técnica.
9. Faz fallback respeitando ou declarando a identidade vocal.
10. Evita mudanças constantes e desnecessárias.
11. Gere versões com rollback e sem substituições não validadas.
12. Opera localmente sem transmitir dados pessoais.
13. Explica decisões e limitações de forma compreensível.
14. Não exige escolha técnica do utilizador para funcionar normalmente.

## 43. Fases de implementação do PIMO

| Fase | Trabalho | Conclusão |
|---|---|---|
| P1 | Manifesto completo e níveis de evidência no catálogo | Todos os candidatos com campos obrigatórios e evidência declarada |
| P2 | Filtro do nível A e Hardware Profiler | Elegíveis e motivos de eliminação visíveis e testados |
| P3 | Validação do nível B com cache | Um modelo real validado, cache invalidada nas mudanças certas |
| P4 | Seleção multicritério e explicação | Escolha estável, explicada e coberta por testes |
| P5 | Monitorização, histerese e recuperação | Sessão de 30 minutos sem oscilação nem perda de texto |
| P6 | Composição de pipeline e fallback de voz | Cadeia completa com degradação declarada |

A ordem é uma dependência, não um calendário. P1 e P2 não precisam de nenhum motor novo; P3 exige o par de tradução resolvido e um clonador medido. A implementação reutiliza o que existe — catálogo, gate de licenças, adaptadores por processo e scripts de avaliação — e não cria um sistema de aprendizado para escolher modelos, porque regras determinísticas sobre manifestos e resultados bastam.
