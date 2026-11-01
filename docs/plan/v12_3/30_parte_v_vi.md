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

A auditoria anterior sugeria que separar processos resolvia a questão GPL. Não resolve: separar processos é uma decisão de engenharia, e a qualificação de obra combinada é uma questão jurídica que este documento não decide. O que o projeto faz é reduzir a exposição e documentar a fronteira com precisão.

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

## 23. Pesos, vozes e datasets

Cada artefacto descarregável tem um registo com: identificador e versão, origem oficial, URL, hash, licença, atribuição obrigatória, condições conhecidas e estado de verificação. O procedimento é o mesmo para todos:

- Abrir a fonte oficial e ler a licença declarada para aquele checkpoint, não para a família.
- Registar o commit ou a revisão exata descarregada.
- Calcular o hash do ficheiro distribuído e guardá-lo no catálogo.
- Registar a licença das vozes ou dos dados em campo próprio, separado do código.
- Marcar como não verificado tudo o que não foi confirmado, e impedir o download pelo catálogo enquanto assim estiver.

Estado atual dos artefactos no catálogo: as três vozes Piper incluídas estão verificadas como MIT no repositório de vozes, o checkpoint Qwen3-ASR 0.6B está verificado como Apache-2.0, e o pacote Argos é o único artefacto ainda marcado como não verificado — precisamente o que está a sair. Nenhum peso de modelo é distribuído dentro de um arquivo de release.

## 24. Modelos bloqueados e catálogo experimental

| Categoria | Comportamento |
|---|---|
| Aprovado | Download em um clique, hash verificado, licença visível |
| Experimental | Área separada, aviso com fonte, licença, desconhecidos e consequência, opt-in explícito |
| Bloqueado | Não aparece para download; aparece a razão na documentação |
| Externo | Instalado fora do Parlo, com contrato de processo e licença própria |

Ficam bloqueados: F5-TTS (pesos CC-BY-NC), XTTS-v2 (licença não comercial), NLLB-200 e SeamlessM4T v2 (CC-BY-NC), e qualquer checkpoint cuja licença não tenha sido confirmada. Ficam experimentais: Qwen3-ASR, VoxCPM, VoiceDesign e diarização com pesos condicionados. Ficam externos: Piper e Argos, se o Argos voltar.

Um modelo bloqueado pode ser usado por quem o instalar por sua conta, mas o Parlo não o descarrega, não o recomenda e não o apresenta como capacidade do produto.

## 25. Direitos de voz e uso aceitável

O Parlo exige o mínimo necessário para ser um projeto responsável, sem transformar a utilização normal num formulário.

| Situação | Exigência |
|---|---|
| A própria voz | Declaração de que a voz é do utilizador |
| Voz de outra pessoa | Declaração de autorização, com finalidade e âmbito, registada localmente |
| Voz de ator, dataset ou voz comercial | Proibida como exemplo ou demonstração no repositório |
| Vozes de exemplo na documentação | Só sintéticas neutras ou do próprio autor do exemplo |
| Vozes pré-instaladas | Não existem |
| Cópia do repositório | Não contém amostras de voz privadas, nem perfis, nem caches |

O registo é uma declaração local, não uma certificação de identidade. O projeto não promete verificar quem é quem; promete não clonar sem declaração, permitir revogar e apagar, e recusar conteúdo que se apresente como outra pessoa para enganar. O âmbito completo está em `ACCEPTABLE_USE.md` e na política de voz publicada no repositório.

## 26. Conformidade automática em cada release

A conformidade deixa de depender de memória. O que existe hoje: um gate que lê a cadeia Rust por `cargo metadata`, a cadeia Python pelas dependências fixadas e o catálogo de modelos, e falha quando uma licença falta, está negada, sai da allowlist do âmbito ou aparece numa dependência sem versão fixa; avisos de terceiros e SBOM CycloneDX gerados do mesmo inventário; índice de licenças regenerado e comparado com o commit; e recusa de build enquanto o gate tiver problemas abertos.

| Controle | Efeito | Estado |
|---|---|---|
| Allowlist por âmbito | GPL só em adaptadores opcionais, nunca no núcleo | Implementado |
| Dependências fixadas | Nenhuma versão flutuante em Rust ou Python | Implementado |
| SBOM CycloneDX por release | Inventário com versões e hashes | Implementado |
| `THIRD_PARTY_NOTICES.md` | Avisos agregados | Implementado |
| `LICENSES.md` validado no CI | Índice coerente com o catálogo | Implementado |
| Verificação de artefactos de modelo | Falha quando falta licença, hash ou estado | Implementado no catálogo; falta bloquear download |
| Hashes de binários de release | Registo por artefacto | Implementado |
| Builds reprodutíveis | Sem dependência do ambiente de quem compila | Parcial: falta fixar toolchain e imagem |

> O gate prova coerência de metadados, não conformidade jurídica. Uma dependência nova com licença aprovada na allowlist pode ainda criar obrigações que exigem leitura humana; por isso a allowlist é revista por componente e as exceções ficam registadas na secção 56, não escondidas num ficheiro de configuração.

# PARTE VI · MODEL HUB, AUTO SELECT E QUALIDADE

## 27. Model Hub

O Model Hub é a peça que transforma um catálogo de ficheiros numa decisão de produto. Conhece o que existe, o que está instalado, o que é compatível com a máquina, o que a licença permite, o que foi medido e o que falhou.

| Responsabilidade | Comportamento |
|---|---|
| Descobrir | Lista por tarefa, idioma, tamanho e estado, sem obrigar a comparar |
| Instalar | Download com hash, progresso, retoma, verificação e espaço em disco |
| Atualizar | Nova versão só quando a licença se mantém e o gate passa |
| Remover | Elimina pesos, caches e representações do modelo, sem tocar em dados do utilizador |
| Explicar | Diz porque escolheu aquele modelo e o que perdeu por não usar outro |
| Recusar | Não instala artefacto sem licença confirmada, sem hash ou incompatível com o hardware |

Requisitos de engenharia: nenhum download silencioso; nenhuma atualização de modelo durante uma sessão ativa; espaço em disco verificado antes de descarregar; e uma pasta de dados por utilizador, nunca dentro do repositório.

## 28. Manifesto por modelo

Cada entrada tem, no mínimo: identificador; nome legível; versão e checkpoint exato; hash; função; idiomas e variantes; runtime e backend; sistemas operativos; suporte de CPU, GPU e NPU; RAM e VRAM estimadas; tamanho; quantização; licença do código; licença dos pesos; licença de vozes ou dados quando aplicável; origem e URL; dependências; estado de validação; capacidades; limitações; regra de recomendação; e estado operacional (recomendado, compatível, experimental, bloqueado, externo).

Campos em falta impedem a entrada no modo automático. O catálogo atual já cobre licença, origem, hash e estado de verificação; cobertura de hardware, latência medida e regra de recomendação é o que falta para o Model Hub existir de facto.

## 29. Auto Select

O algoritmo é determinístico e explicável. Ordem de decisão: eliminar o que é incompatível ou está bloqueado; filtrar por tarefa e idioma, distinguindo variantes regionais; filtrar por recursos reais disponíveis; ordenar por qualidade medida e latência medida; desempatar por consumo e estabilidade; escolher e registar a razão.

| Sinal | Uso |
|---|---|
| Tarefa | Reconhecimento, tradução, clonagem, síntese neutra, diarização, alinhamento |
| Idioma e variante | PT-PT nunca tratado como PT-BR nem como «pt» genérico |
| Hardware | Arquitetura, instruções disponíveis, memória livre, armazenamento |
| Backend | CPU, GPU, NPU e runtime efetivamente utilizável |
| Licença | Exclui o que não pode entrar no perfil de distribuição escolhido |
| Qualidade medida | Resultados internos por tarefa e par de línguas |
| Latência medida | Por perfil de hardware e por modo de utilização |
| Estado | Prefere recomendado; aceita compatível; nunca escolhe experimental por omissão |
| Energia | Preferência por menor consumo quando a qualidade é equivalente |

O utilizador vê Automático. Quando o Auto Select troca de motor, a interface diz qual passou a usar e porquê, em uma frase. Um override manual é permitido, mas fixa a escolha e passa a responsabilidade de compatibilidade para o utilizador, com aviso claro.

## 30. Fallback

| Domínio | Cadeia | Aviso ao utilizador |
|---|---|---|
| Reconhecimento | Motor preferido, motor compatível, modelo mais leve, ditado desativado com aviso | Sim, quando muda de motor |
| Tradução | Par direto, par alternativo aprovado, pivô por inglês, só transcrição | Sim, no pivô e na ausência de tradução |
| Clonagem | Clonador preferido, clonador alternativo com o mesmo perfil, voz neutra, só texto | Sim, obrigatório a partir da voz neutra |
| Síntese neutra | Voz do locale, voz de locale próximo, leitura desativada | Sim |
| Tempo real | Modo incremental, modo por blocos, só legendas | Sim, com o atraso esperado |
| Reuniões | Captura de microfone, só importação de gravação, transcrição manual | Sim |
| Dobragem | Faixa completa, legendas, áudio original com notas | Sim |

Regras: o fallback nunca perde texto nem áudio já capturado; a troca é registada no histórico da sessão; a interface nunca apresenta o resultado de um fallback como se fosse o caminho principal; e se nenhum candidato preservar a voz, o produto diz que a identidade vocal não pode ser garantida naquele modo.

## 31. Benchmarks internos e gates

Os benchmarks são executados pelo projeto, com corpus versionado, hardware declarado e resultados guardados. Não são pedidos ao utilizador e não são substituídos por leaderboards.

| Dimensão | Métrica | Gate |
|---|---|---|
| Reconhecimento | WER por corpus e variante; primeiro parcial; texto final | WER não piora face à release anterior no mesmo corpus |
| Tradução | Fidelidade humana, preservação de números, negações e nomes, latência | Conjunto representativo, sem regressões, falhas explicadas |
| Clonagem | Similaridade de locutor, inteligibilidade da saída, naturalidade cega | Limiar interno definido por motor e par de línguas |
| Síntese | Tempo até ao primeiro áudio, estabilidade em sessão longa | Orçamento por modo de utilização |
| Recursos | RSS, VRAM, CPU, energia por minuto de áudio | Cabe no perfil de hardware declarado |
| Estabilidade | Sessão de 30 minutos sem deriva de fila nem crescimento de memória | Zero falhas críticas |
| Privacidade | Auditoria de rede durante a sessão | Zero ligações externas |

Um resultado só é publicado com hardware, versões, quantização, threads, corpus e data. Números antigos ficam como histórico e não como benchmark atual. A regra que fecha esta parte: nenhuma afirmação de superioridade sobre outro projeto sem teste comparável, e nenhum modelo promovido a default por leitura de documentação.
