# PARTE I · FUNDAMENTOS

## 1. Estado do projeto e decisões executivas

O Parlo tem hoje um núcleo funcional pequeno e verificável: reconhecimento local, tradução local e síntese opcional, ligados por um protocolo local, mais uma aplicação de janela própria. Tem também uma política de licenciamento implementada e automatizada. Não tem clonagem de voz, não tem Model Hub, não tem seleção automática e não tem a tradução EN→PT resolvida. Esta secção diz exatamente onde está cada coisa, porque um plano que confunde protótipo com produto leva a decisões erradas nas partes seguintes.

### 1.1 Estado verificado nesta revisão

| Área | Estado | Evidência |
|---|---|---|
| Última tag publicada | v0.1.6 | `git tag` no repositório; o CHANGELOG tem entradas até 0.1.7 |
| Motor de cascata | Implementado | `engines/cascade` reconhece, traduz e sintetiza por blocos, em CPU |
| Reconhecimento | Implementado e medido | Parakeet TDT v3 int8 por sherpa-onnx e Whisper por faster-whisper |
| Estados de texto | Implementado | Parcial, estável, final e corrigido; sessão fecha após 0,45 s de silêncio |
| Tradução | Implementada com dependência em revisão | Argos Translate no caminho publicado, com MiniSBD AGPL transitivo |
| Tradução OPUS-MT | Implementada em laboratório, não publicada | PT→EN correto; EN→PT degenera nos dois checkpoints testados |
| Síntese | Opcional e externa | Adaptador Piper separado, GPL-3.0-or-later, fora do programa do motor |
| Clonagem de voz | Não existe | Sem Voice Profile, sem consentimento, sem motor de clonagem |
| Model Hub e Auto Select | Não existem | Existe catálogo de licenças e painel de download, sem seleção automática |
| Diarização e Meeting | Não existem | Sem código, sem captura multifonte |
| Licenciamento | Implementado | Apache-2.0 no código original, MIT preservado para releases antigas, avisos e SBOM gerados |
| Gate de licenças | Implementado | `tools/licenses.py` lê a cadeia Rust, a cadeia Python e o catálogo de modelos |

### 1.2 Números medidos, com condições

| Medição | Valor | Condições |
|---|---|---|
| Primeiro parcial PT-PT | 849 ms | Parakeet TDT v3 int8, 4 threads, CPU, 20 frases |
| Texto final PT-PT | 3,36 s | Mesma execução, após fecho do segmento |
| Frases confirmadas antes do fecho | 81% | Mesma execução |
| WER final PT-PT | 0,231 Parakeet · 0,279 Whisper base | Corpus PT-PT de 20 frases, voz Piper europeia |
| Tradução PT→EN por OPUS-MT | 69 a 130 ms por frase | CTranslate2 int8, 4 threads, beam 4, penalização de repetição |
| Cobertura de licenças | 523 componentes, zero problemas | `tools/licenses.py check --verify-installed` |

Estes números não são promessas de produto. São o ponto de partida dos gates da secção 31 e mudam quando o corpus, o modelo ou o hardware mudarem.

### 1.3 Decisões executivas

| Tema | Decisão |
|---|---|
| Experiência principal | Falar na própria voz e ouvir a tradução na própria voz, quando o hardware e um motor aprovado o permitirem |
| Voz neutra | Existe como fallback explícito, teste de integridade e escolha consciente; nunca substitui a voz da pessoa em silêncio |
| Escolha de modelos | O Parlo decide; o utilizador vê Automático. A escolha manual é override avançado |
| Tradução | NMT por par com checkpoint aprovado; nenhum par é declarado suportado sem avaliação interna |
| Licença do código | Apache-2.0 para o código original; MIT preservado nas releases antigas |
| Componentes externos | Piper, Argos e qualquer runtime copyleft ficam fora do núcleo e fora dos arquivos de release |
| Pesos e vozes | Descarregados à parte, nunca incluídos no instalador, cada um com a sua licença registada |
| Contribuições | DCO, sem transferência de direitos, com regra de proveniência |
| Plataforma inicial | Windows primeiro; macOS e Linux por marcos próprios |
| Privacidade | Áudio, perfis, transcrições e traduções ficam no dispositivo; nenhuma telemetria obrigatória |

> Regra de leitura do estado: enquanto a secção 55 tiver itens abertos, o Parlo não anuncia a capacidade correspondente. Uma capacidade em avaliação não aparece no README, nas notas de release nem na interface como se existisse.

## 2. Visão, público e família de produtos

**Visão.** Uma pessoa fala na sua língua, com a sua voz, e é ouvida na língua do interlocutor, sem que o áudio saia do computador. O Parlo não substitui intérpretes profissionais; retira a barreira técnica e a dependência de serviços exteriores nos casos em que uma pessoa precisa de continuar a falar como ela própria.

A voz é parte do argumento, não um detalhe de acabamento. Uma tradução falada numa voz genérica faz o interlocutor ouvir outra pessoa. O Parlo persegue a identidade vocal autorizada e diz com clareza quando não a consegue garantir.

### 2.1 Público

| Público | Necessidade | Superfície |
|---|---|---|
| Profissionais que trabalham em língua estrangeira | Escrever e falar com correção sem perder tempo | Dictate e Traduzir seleção |
| Equipas internacionais em chamadas | Acompanhar e intervir na própria voz | Captions e Live |
| Pessoas com perda auditiva ou em ambientes ruidosos | Legendas fiáveis e legíveis | Captions |
| Utilizadores com requisitos de privacidade | Nada sai da máquina | Todos os modos, por defeito |
| Criadores de conteúdo | Dobrar conteúdo autorizado na própria voz | Dub |
| Programadores e integradores | Contratos estáveis, CLI e API | CLI, API local, SDK |

### 2.2 Família de produtos sobre o mesmo núcleo

| Produto | Utilidade | Estado |
|---|---|---|
| Parlo Dictate | Ditado local com escrita noutra língua | Parcial: motor e CLI existem, experiência de overlay por fechar |
| Parlo Captions | Legendas do microfone e do áudio do sistema | Parcial: existe modo de legendas na cascata, sem captura de sistema |
| Parlo Voice | Leitura de texto e de traduções | Parcial: Piper externo opcional; clonagem não existe |
| Parlo Live | Interpretação bidirecional falada | Planeado |
| Parlo Meeting | Reuniões com transcrição e chat | Planeado |
| Parlo Dub | Dobragem de conteúdo autorizado | Experimental, fora do caminho crítico |
| Parlo CLI e API | Integração local | CLI implementada; API local estável planeada |

**Posicionamento honesto.** O Parlo compete com ferramentas de ditado e gestores de modelos, não com serviços de interpretação simultânea. O que pode ser genuinamente melhor é a combinação de voz própria, execução local, gestão de modelos sem escolha técnica e transparência de licenças. Nenhuma dessas vantagens está demonstrada enquanto não houver produto e medições públicas.

## 3. Princípios não negociáveis

- **Local por defeito.** Nenhuma chamada de rede durante uma sessão local, incluindo telemetria, analytics ou atualização silenciosa. A verificação de atualizações é a única chamada permitida, e não transporta áudio, identificadores ou conteúdo.
- **Voz da pessoa primeiro.** O caminho principal tenta clonagem autorizada antes de qualquer voz neutra. A degradação para voz genérica é explícita e visível.
- **Incremental por desenho.** O sistema processa áudio enquanto a pessoa fala e expõe texto provisório e estável, sem esperar pelo fim da frase.
- **Não inventar simultaneidade.** Se um modelo só funciona por blocos, a interface diz isso e não vende streaming que não existe.
- **Sem bloqueio de fornecedor.** Modelos e runtimes são substituíveis por adaptadores com contrato versionado.
- **Sem licença escondida.** A licença de cada artefacto aparece antes do download e antes da ativação.
- **Qualidade honesta.** Latência, precisão, memória e compatibilidade são publicadas com hardware, versões e método.
- **Degradação graciosa.** Se a voz falhar, mantêm-se legendas e texto; nunca se perde o que a pessoa disse.
- **Consentimento verificável.** Nenhum perfil de voz nasce sem declaração explícita de autorização, e todos podem ser apagados por completo.
- **Código e pesos são artefactos distintos.** Uma licença permissiva no código nunca se estende automaticamente a um checkpoint, a uma voz ou a um dataset.

# PARTE II · VOZ COMO CAPACIDADE CENTRAL

## 4. Política Voice Clone First

A síntese deixou de ser um módulo de leitura em voz alta para se tornar o centro do produto. Esta política define o que o Parlo tenta primeiro, o que faz quando não consegue, e o que diz ao utilizador.

### 4.1 Ordem de tentativa

| Ordem | Caminho | Quando é usado | O que o utilizador vê |
|---|---|---|---|
| 1 | Clonagem autorizada com o perfil do falante | Existe Voice Profile consentido e um motor aprovado para o par de línguas e o hardware | «A falar na tua voz» |
| 2 | Clonagem autorizada com outro motor aprovado | O motor preferido falhou, excedeu memória ou não cobre o idioma | «A falar na tua voz com outro motor» |
| 3 | Voz neutra aprovada para o locale | Nenhum clonador aprovado serve a combinação | «A falar com uma voz neutra, porque a tua voz não está disponível neste idioma» |
| 4 | Só texto e legendas | Síntese indisponível ou desativada | «Sem voz: mostro o texto traduzido» |

A ordem é aplicada pelo Auto Select, não pelo utilizador. A troca nunca é silenciosa: os estados 3 e 4 exigem aviso visível e registo no histórico da sessão.

### 4.2 O que a política proíbe

- Substituir a voz da pessoa por uma voz genérica sem o dizer, mesmo quando a qualidade sobe.
- Anunciar «voz preservada» quando o motor mudou e a identidade não foi medida nesse motor.
- Prometer clonagem em tempo real em hardware onde não foi medida.
- Clonar voz de terceiros sem autorização declarada, ou manter perfis de voz de terceiros pré-instalados.
- Usar amostras de voz para treino, telemetria ou melhoria de modelos. Não existe recolha de dados para treino.

### 4.3 Consequências para o resto do plano

- A matriz de modelos da secção 20 distingue quem clona, quem só sintetiza e quem desenha voz.
- O Model Hub da secção 27 tem a clonagem como tarefa de primeira classe, com requisitos de memória e latência próprios.
- O gate de qualidade da secção 31 inclui identidade vocal, não apenas inteligibilidade.
- A interface da secção 11 distingue três estados de voz e nunca os confunde.

## 5. Voice Profile, consentimento e ciclo de vida

Um Voice Profile é um recurso local que representa a voz de uma pessoa autorizada. Não é um ficheiro solto no disco: tem identidade, finalidade, consentimento, artefactos derivados e uma operação de eliminação que remove tudo.

### 5.1 Criação

| Passo | O que acontece | Regra |
|---|---|---|
| 1. Escolha do método | Gravar agora ou importar áudio | Importação exige declaração de autorização sobre o ficheiro |
| 2. Recolha | 10 a 30 s de fala contínua, ou mais quando o motor exigir | O produto indica o mínimo do motor escolhido, não um número universal |
| 3. Controlo de qualidade | Nível, ruído de fundo, eco, presença de uma só voz, fala suficiente | Rejeita ou pede nova amostra com uma razão concreta |
| 4. Texto de referência | Transcrição automática da amostra, corrigível | Obrigatório quando o motor precisa de texto alinhado |
| 5. Consentimento | Declaração explícita com finalidade e âmbito | Sem consentimento não há perfil |
| 6. Pré-visualização | Frase curta sintetizada com o perfil | O utilizador ouve antes de ativar |
| 7. Ativação | Perfil guardado e disponível para os motores compatíveis | Nome, idioma de referência e data visíveis |

### 5.2 Registo de consentimento

O registo é local e contém, no mínimo: identificador do perfil; declaração de que a voz é do próprio ou de pessoa que autorizou; finalidade (ditado, legendas, reuniões, dobragem); versão do texto de informação apresentado; data e hora; e o âmbito autorizado. Não se inventa verificação de identidade que o projeto não consegue garantir: o Parlo registra uma declaração, não certifica uma pessoa.

### 5.3 Proteção e armazenamento

- Amostras, embeddings, prompts de voz e cache vivem numa pasta de dados do utilizador, fora do repositório e fora dos arquivos de release.
- Encriptação opcional com chave do sistema operativo (DPAPI no Windows, Keychain no macOS, Secret Service no Linux), ativada por pedido do utilizador e documentada.
- Nada é enviado para fora da máquina, nem para verificação, nem para melhoria de qualidade.
- Perfis de terceiros pré-instalados não existem, em nenhuma circunstância.

### 5.4 Eliminação

A eliminação de um perfil remove amostra de referência, transcrição de referência, embeddings, prompts, caches, previews e registos de consentimento associados, e invalida qualquer cache de síntese que dependa dele. A operação termina com uma confirmação que lista o que foi apagado. Exportar e importar perfis é permitido, mas o ficheiro exportado é marcado como sensível e não é incluído em cópias automáticas.

## 6. Compatibilidade de perfis entre motores

Um perfil criado para um motor não é um perfil universal. Motores diferentes usam representações diferentes: alguns guardam embeddings, outros precisam de áudio de referência com transcrição, outros de um prompt acústico com duração fixa. Assumir compatibilidade é a forma mais rápida de quebrar a promessa de identidade vocal.

### 6.1 Modelo de dados

| Camada | Conteúdo | Partilhada? |
|---|---|---|
| Identidade | Nome, idioma de referência, consentimento, data | Sim, entre motores |
| Amostra de referência | Áudio autorizado e transcrição corrigida | Sim, se o utilizador autorizar a sua preservação |
| Representação por motor | Embedding, prompt ou cache específico do motor e da versão | Não: é derivada e versionada |
| Cache de síntese | Áudio gerado, por texto e motor | Não: descartável |

### 6.2 Regras de mudança de motor

- O Parlo verifica se existe representação válida para o motor de destino. Se existir, usa-a; se não, deriva-a a partir da amostra de referência.
- Se a amostra de referência foi apagada por opção do utilizador, a derivação é impossível. O Parlo diz isso antes da troca, não depois, e oferece uma de duas saídas: manter o motor atual ou criar um novo perfil com nova gravação.
- Mudar de motor não autoriza anunciar a mesma qualidade. A identidade vocal do novo motor só é afirmada depois de medida segundo a secção 7.
- Cada representação guarda o identificador e a versão do motor que a produziu. Uma atualização de motor que invalide a representação é tratada como incompatibilidade, não como erro genérico.

## 7. Identidade vocal entre idiomas

Preservar a voz entre idiomas não significa preservar o sotaque. O objetivo é manter o timbre, a qualidade da voz e traços prosódicos reconhecíveis, aceitando que a pronúncia de uma língua estrangeira pode ser marcada pela língua de origem. A interface deve dizer isto de forma simples, porque é a expectativa mais comum e a mais fácil de desiludir.

### 7.1 O que é medido

| Dimensão | Método | Uso |
|---|---|---|
| Semelhança de identidade | Similaridade de embedding de locutor entre referência e saída | Gate interno de clonagem |
| Inteligibilidade | Taxa de erro de reconhecimento da saída sintetizada | Gate de utilidade |
| Naturalidade | Avaliação cega por ouvintes, escala curta | Gate de qualidade percebida |
| Estabilidade | Variação entre frases e ao longo de sessões longas | Gate de sessões prolongadas |
| Latência | Tempo até ao primeiro áudio sintetizado | Gate de modo quase-tempo-real |
| Cobertura por idioma | Resultado por par de línguas e variante | Decisão do Auto Select |

Os limiares numéricos são internos e publicados apenas com o método e o hardware ao lado. Um valor sem condições não entra em notas de release.

### 7.2 Limites assumidos

- A identidade mantém-se tanto melhor quanto mais próxima for a língua de destino da língua da amostra. Pares distantes exigem amostras maiores e produzem resultados mais frágeis.
- Nenhum motor é declarado bom em todos os pares. O estado é por combinação: motor, par de línguas, variante regional e hardware.
- Sotaque, ritmo e hesitações não são reproduzidos por igual em todos os motores; alguns normalizam a prosódia e perdem espontaneidade.
- Em hardware modesto, a clonagem pode não cumprir o orçamento de latência. Nesse caso aplica-se a secção 4.1, sem promessas.

## 8. Síntese neutra, desenho de voz e conversão de voz

Três funcionalidades vizinhas da clonagem, com papéis diferentes e sem se confundirem com ela.

| Capacidade | O que é | Papel no produto | Regra |
|---|---|---|---|
| Síntese neutra | Voz genérica de catálogo, sem identidade do utilizador | Fallback, leitura de texto, verificação de integridade do pipeline | Sempre identificada como voz neutra |
| Desenho de voz | Geração de uma voz a partir de uma descrição, sem amostra de uma pessoa | Criação de vozes para conteúdo próprio, testes, acessibilidade | Não é clonagem; não usa perfil de pessoa real |
| Conversão de voz | Transformação de áudio existente para outra identidade vocal | Só quando trouxer vantagem medida sobre a síntese com clonagem | Exige autorização sobre a voz de destino e sobre o áudio |
| Clonagem | Síntese com a identidade de uma pessoa autorizada | Caminho principal | Consentimento obrigatório |

A conversão de voz acrescenta uma etapa ao pipeline, aumenta a latência, potencia artefactos e cria uma superfície adicional de direitos. Fica fora do caminho crítico e só entra se a comparação interna mostrar vantagem clara em qualidade ou em custo, com a mesma autorização exigida para clonagem.

A voz neutra continua a ter função: valida a saúde do pipeline quando a clonagem falha, serve quem não quer clonar, e permite ouvir uma tradução em minutos numa máquina onde nenhum clonador cumpre o orçamento. Nunca é o resultado por defeito quando existe perfil válido.
