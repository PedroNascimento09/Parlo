# PARTE VII · ARQUITETURA E PLATAFORMA

## 44. Camadas, núcleo e isolamento

| Camada | Responsabilidade | Tecnologia |
|---|---|---|
| Aplicação | Janela, overlays, estado visível, preferências | Tauri 2 com interface local |
| Núcleo | Sessões, captura, orquestração, inserção de texto, política de modelos | Rust |
| Motor | Reconhecimento, tradução, síntese, diarização | Processos Python isolados |
| Adaptadores | Um por família de modelos, com licença e contrato próprios | Processos separados |
| Catálogo | Manifestos, licenças, hashes, estado de validação | Ficheiros versionados |

Regras de isolamento: um adaptador que falha não derruba a sessão; o núcleo aplica tempo limite e substitui o motor; nenhum adaptador escreve diretamente em dados do utilizador; e cada adaptador declara a sua licença no manifesto, para que a fronteira de distribuição seja auditável.

## 45. Protocolo entre núcleo e motores

O transporte é JSON por linha sobre pipes do processo, com identificador de pedido, operação, versão de protocolo e resposta ou erro estruturado. Operações atuais: saúde, aquecimento, tradução por bloco, streaming de texto e dobragem.

Contrato: campos obrigatórios validados dos dois lados; versão de protocolo negociada no arranque; erro sempre com código e mensagem legível; nenhum adaptador pode enviar áudio para a rede; e o núcleo recusa um adaptador que declare uma licença incompatível com o perfil de distribuição ativo. Uma alteração incompatível do protocolo exige versão nova e adaptadores atualizados, com teste de contrato no CI.

## 46. Pipeline de áudio, latência e adaptação

Caminho do áudio: captura do dispositivo, verificação de formato, deteção de atividade, reconhecimento incremental, estabilização de texto, tradução por unidades semânticas, síntese ou legendas, e saída para o dispositivo, para o campo ativo ou para ficheiro.

| Etapa | Orçamento no perfil equilibrado | Como é medido |
|---|---|---|
| Captura a primeiro parcial | 1,5 s | Timestamp do primeiro evento parcial menos início de fala |
| Primeiro parcial a texto estável | 1,0 s | Evento estável contra o parcial anterior |
| Texto estável a tradução | 400 ms | Tempo de tradução por unidade |
| Tradução a primeiro áudio sintetizado | 1,2 s | Soma de síntese e reamostragem |

Os valores atuais medidos são: 849 ms até ao primeiro parcial e 3,36 s até ao texto final em PT-PT. O orçamento de síntese ainda não tem medição porque nenhum clonador está integrado. O controlador adaptativo reduz o tamanho das unidades, muda o modo de processamento ou desativa a voz quando o atraso acumula, e diz ao utilizador o que mudou.

## 47. Hardware, backends e perfis

| Perfil | Hardware típico | O que se espera | O que não se promete |
|---|---|---|---|
| Mínimo | 4 núcleos, 8 GB, sem GPU utilizável | Reconhecimento e tradução funcionam; voz neutra leve | Clonagem em tempo real |
| Equilibrado | 8 núcleos, 16 GB, iGPU | Reconhecimento rápido, tradução local, clonagem em blocos | Latência de conversa |
| Alto | 16 núcleos, 32 GB, GPU dedicada | Clonagem com boa qualidade, quase-tempo-real | Paridade com estúdio |
| Workstation | 64 GB e GPU de grande memória | Qualidade máxima e sessões longas | Suporte universal |

A seleção não usa regras simplistas. Cada perfil é uma combinação medida de memória livre, instruções do processador, backend disponível e resultado do benchmark. Um computador com 8 GB mas com GPU utilizável não é tratado como o perfil mínimo.

## 48. Extensibilidade: CLI, API, SDK e plugins

| Superfície | Estado | Contrato |
|---|---|---|
| CLI | Implementada | Comandos estáveis, saída legível e código de saída definido |
| API local | Existe servidor em loopback com token | Rota documentada, sem promessa de estabilidade antes da v1.x |
| SDK | Planeado | Contratos versionados e testes de compatibilidade |
| Plugins | Planeado | Permissões granulares e níveis oficial, comunitário e experimental |
| MCP | Planeado | Opcional, com autorização explícita por ferramenta |

Regra: uma capacidade só chega à API depois de existir na aplicação. A aplicação nunca depende de uma interface que o utilizador não vê, e nenhuma superfície de automação recebe acesso a perfis de voz sem autorização explícita.

# PARTE VIII · PRIVACIDADE, SEGURANÇA E LEGISLAÇÃO

## 49. Privacidade local-first

| Dado | Onde vive | Por defeito |
|---|---|---|
| Áudio capturado | Memória da sessão | Não é guardado sem pedido |
| Amostra de referência de voz | Pasta de dados do utilizador | Guardada só com consentimento |
| Embeddings e prompts de voz | Pasta de dados do utilizador | Derivados e apagáveis |
| Transcrições | Pasta de dados do utilizador | Guardadas só com histórico ativo |
| Traduções | Sessão e histórico opcional | Apagáveis |
| Gravações de reunião | Pasta de dados do utilizador | Só com aviso e opt-in |
| Caches | Pasta de dados do utilizador | Apagáveis em conjunto |
| Registos de diagnóstico | Pasta de dados do utilizador | Sem áudio por omissão |
| Relatórios de erro | Nunca enviados automaticamente | Envio manual e revisto |
| Analytics e telemetria | Não existem | — |

A única chamada externa permitida é a verificação da última release estável, e não transporta áudio, conteúdo, identificadores de utilizador nem telemetria. Nenhum áudio sai da máquina em modo local. Nenhum dado é recolhido para treino.

## 50. Dados pessoais e biométricos

Um embedding de voz não é automaticamente dado biométrico de categoria especial em todos os contextos: depende da finalidade e do tratamento. Uma representação usada para identificar uma pessoa é tratada como dado biométrico; uma representação usada para gerar áudio na voz dessa pessoa é, no mínimo, dado pessoal e exige controlo reforçado. Esta distinção é técnica e não substitui avaliação jurídica.

Medidas que o Parlo adota independentemente da qualificação: base legal assente em consentimento para o perfil de voz; finalidade declarada e limitada; minimização (não guardar áudio se bastar a representação derivada, quando o motor o permitir); prazo de conservação definido pelo utilizador; eliminação completa; portabilidade por exportação; e nenhuma partilha com terceiros.

## 51. Regulamento Europeu da IA e Artigo 50

O Parlo é um projeto open-source, gratuito e local. Isso reduz exposição, mas não elimina automaticamente obrigações: a isenção para sistemas de código aberto não cobre todos os casos, e as obrigações de transparência do Artigo 50 aplicam-se a partir de 2 de agosto de 2026. As datas e a interpretação devem ser confirmadas com aconselhamento jurídico antes de qualquer declaração pública de conformidade.

| Obrigação | A quem se aplica | O que o Parlo faz |
|---|---|---|
| Informar que se interage com um sistema de IA | Fornecedor que coloca o sistema no mercado | A aplicação diz que a voz é sintetizada, sempre que o for |
| Marcar conteúdo sintético de forma legível por máquina | Fornecedor de sistemas generativos | Metadados de origem e marca de água quando tecnicamente viável; documentado o que falta |
| Divulgar deepfakes | Responsável pela utilização que publica o conteúdo | O produto avisa e regista; a divulgação é responsabilidade de quem publica |
| Transparência sobre limitações | Fornecedor | Documentação de limitações por modo e por idioma |

Regras de produto: a saída sintetizada é sempre identificada na interface e nos ficheiros exportados; o utilizador recebe um aviso claro de que a divulgação de conteúdo que imita uma pessoa pode ser exigida por lei; e nenhuma funcionalidade de voz é apresentada como indistinguível de uma gravação real. Não se afirma que toda a tradução de voz é um deepfake: a classificação depende do contexto e da forma como o conteúdo é apresentado.

## 52. Segurança e cadeia de fornecimento

- Downloads apenas por HTTPS de origem declarada no manifesto, com hash verificado antes do uso.
- Nenhum modelo executado a partir de um caminho arbitrário sem verificação de integridade.
- Modelos tratados como dados não fidedignos: processamento isolado, sem acesso a ficheiros do utilizador.
- Instaladores assinados quando existir certificado; caso contrário, publicação de hashes e instruções de verificação.
- Dependências fixadas, SBOM por release e revisão de novas dependências antes de entrarem no núcleo.
- Nenhuma credencial no repositório; verificação de segredos no CI.
- Relato de vulnerabilidades com prazo de resposta declarado em `SECURITY.md`.

## 53. Reuniões: consentimento e contexto

Gravar uma reunião exige aviso e, em muitos contextos, consentimento dos participantes. O Parlo não decide isso pelo utilizador: mostra um aviso claro antes da primeira gravação, exige confirmação por sessão, oferece o modo «sem guardar áudio» e permite apagar uma sessão inteira com os seus artefactos. Nenhuma reunião é enviada para fora da máquina, nem resumida por um serviço externo.

# PARTE IX · REUNIÕES

## 54. Parlo Meeting: Lite, Live e Chat

Três entregas distintas, com dependências próprias. Durante uma reunião, o Parlo deve ser quase invisível quando funciona e imediatamente claro quando precisa de atenção.

| Entrega | O que faz | Dependências | Estado |
|---|---|---|---|
| Meeting Lite | Importar gravação e produzir transcrição com timestamps, falantes corrigíveis e exportação | Reconhecimento, diarização offline, modelo de dados | Planeado |
| Meeting Live | Transcrever em direto com microfone e áudio do sistema | Captura multifonte, diarização, avisos, estabilidade | Planeado |
| Meeting Chat | Perguntar a reuniões autorizadas com citações e timestamps | Indexação local, abstenção, proveniência | Planeado |

Requisitos transversais: diarização sem identificação nominal automática; associação falante-pessoa apenas manual; resumo e tarefas derivados da transcrição com citação; exportação em Markdown, SRT, VTT e JSON; e eliminação que remove áudio, transcrição, índices e caches. Nenhuma análise pós-reunião pode bloquear a transcrição em direto.
