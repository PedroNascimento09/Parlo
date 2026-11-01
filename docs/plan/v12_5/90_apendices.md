# APÊNDICES

## A. Factos por componente

Verificação documental realizada em 1 de novembro de 2026. Cartões e repositórios mudam; antes de distribuir, fixar commit ou checkpoint, descarregar a licença exata e registar o hash.

| Componente | Função | Licença do código | Licença de pesos ou vozes | PT-PT | Estado |
|---|---|---|---|---|---|
| sherpa-onnx 2.0.1 | Runtime de reconhecimento | Apache-2.0 | Não aplicável | Sim | Em uso |
| faster-whisper 1.2.1 | Runtime Whisper | MIT | Não aplicável | Sim | Em uso |
| CTranslate2 | Runtime de tradução e de Whisper | MIT | Não aplicável | Sim | Aceite |
| Parakeet TDT 0.6B v3 | Reconhecimento | Apache-2.0 | CC-BY-4.0 | Sim | Recomendado |
| Whisper large-v3-turbo | Reconhecimento | MIT | MIT | Sim | Compatível |
| Qwen3-ASR 0.6B | Reconhecimento | Apache-2.0 | Apache-2.0 | Por confirmar | Experimental |
| Argos Translate 1.11.0 | Tradução | MIT | Pacotes por verificar | Sim | Em revisão |
| MiniSBD | Segmentação de frases | AGPLv3 | Não aplicável | Não aplicável | Risco: sair do núcleo |
| OPUS-MT ROMANCE-en | Tradução PT→EN | MIT (Opus-MT) | Apache-2.0 | Sim, com prefixo vazado em 5 de 12 | Candidato, adaptador por limpar |
| OPUS-MT tc-big-en-pt | Tradução EN→PT | MIT (Opus-MT) | CC-BY-4.0 | Sim no corpus, com falhas de variante | Candidato prioritário; conversão por corrigir |
| M2M100 418M | Tradução | MIT | Por confirmar | Por medir | Candidato |
| Qwen3-TTS 0.6B Base | Clonagem | Apache-2.0 | Apache-2.0 | Por medir | Candidato prioritário |
| Qwen3-TTS 1.7B Base | Clonagem | Apache-2.0 | Apache-2.0 | Por medir | Candidato |
| Qwen3-TTS 1.7B CustomVoice | Voz predefinida | Apache-2.0 | Apache-2.0 | Por medir | Compatível, não clona |
| Qwen3-TTS 1.7B VoiceDesign | Desenho de voz | Apache-2.0 | Apache-2.0 | Por medir | Experimental |
| Chatterbox Multilingual pt-PT | Clonagem PT-PT | MIT | MIT | Sim | Candidato prioritário |
| Chatterbox Multilingual pt-BR | Clonagem pt-BR | MIT | MIT | Não | Catálogo separado |
| VoxCPM2 e VoxCPM 0.5B | Clonagem | Apache-2.0 | Apache-2.0 | Por medir | Experimental |
| Kokoro 82M | Voz neutra | Apache-2.0 | Apache-2.0 | Não | Compatível |
| Piper / rhasspy voices | Voz neutra | piper1-gpl GPL-3.0-or-later | MIT no repositório de vozes | Sim | Externo |
| espeak-ng | Fonemização do Piper | GPL-3.0 | Não aplicável | Sim | Externo |
| F5-TTS | Clonagem | MIT | CC-BY-NC | Por medir | Bloqueado |
| XTTS-v2 | Clonagem | Coqui Public Model License | Não comercial | Sim | Bloqueado |
| NLLB-200 | Tradução | MIT | CC-BY-NC-4.0 | Sim | Bloqueado |
| SeamlessM4T v2 | Tradução e voz | MIT | CC-BY-NC-4.0 | Sim | Bloqueado |
| Silero VAD | Deteção de voz | MIT | Não aplicável | Não aplicável | Recomendado |
| pyannote | Diarização | MIT (código) | Condições de acesso | Sim | Candidato condicionado |
| PyInstaller | Empacotamento | GPL com exceção de uso | Não aplicável | Não aplicável | Aceite com aviso |

## B. Glossário operacional

- **Voice Clone First** — política que tenta primeiro a voz autorizada da pessoa e trata a voz neutra como recurso.
- **Voice Profile** — recurso local com identidade, amostra de referência, representações por motor e consentimento.
- **Representação** — artefacto derivado de um perfil para um motor específico, como embedding ou prompt de voz.
- **Clonagem** — síntese com a identidade de uma pessoa autorizada, a partir de amostra.
- **Desenho de voz** — criação de uma voz por descrição, sem amostra de pessoa real.
- **Voz neutra** — voz de catálogo sem identidade do utilizador.
- **Model Hub** — componente que conhece, instala, verifica e explica modelos.
- **Auto Select** — algoritmo determinístico que escolhe o modelo por tarefa, idioma, hardware e licença.
- **Gate** — condição objetiva que uma release ou um modelo tem de cumprir para avançar.
- **Recommodado, compatível, experimental, bloqueado, externo** — estados operacionais do catálogo.
- **Par validado** — par de línguas com avaliação humana e corpus, não apenas cobertura declarada.
- **Pivô** — tradução por uma língua intermédia, sempre assinalada.
- **Declaração de consentimento** — registo local com finalidade, âmbito, versão do aviso e data.
- **Fronteira de licença** — separação técnica entre núcleo permissivo e componentes com outra licença; não é, por si só, conclusão jurídica.
- **PIMO** — orquestrador que filtra, avalia, seleciona, compõe e monitoriza modelos com regras determinísticas.
- **Níveis A, B e C** — avaliação estática por manifesto, validação técnica no dispositivo e avaliação aprofundada de qualidade.
- **Histerese** — exigência de que a alternativa seja claramente melhor para justificar troca de motor.
- **Circuit breaker** — retirada temporária de um motor da seleção após falhas repetidas.
- **Chave de cache** — combinação de modelo, hash, runtime, quantização, hardware e versão do benchmark que valida um resultado.
- **Ilha de ditado** — janela pequena, sem moldura, sempre acima das outras, que mostra o estado de escuta e o texto final sem roubar o foco.
- **Tokens de design** — valores nomeados de cor, tipografia, espaço, raio, elevação e movimento; qualquer valor fora deles é um erro de implementação.
- **Wireframe** — esquema de baixa fidelidade que fixa disposição, hierarquia e estados de um ecrã antes de qualquer código.
- **Recomendação** — configuração escolhida pelo PIMO para uma tarefa, idioma e dispositivo; ponto de partida reversível, nunca bloqueio.
- **Marcador de falante** — identificador genérico de uma voz na reunião (`Speaker 2`), distinto de uma pessoa identificada.
- **Nomeação retroativa** — aplicação de um nome inferido a todos os segmentos do mesmo marcador, mantendo o marcador original para auditoria.
- **Ask Parlo** — conversa sobre uma reunião, com referências obrigatórias ao transcript.
- **Meeting Memory** — camada local que guarda transcript, pessoas, decisões, tarefas, tópicos e relações para responder a perguntas entre reuniões.

## C. Fontes primárias

- Handy — projeto e documentação de modelos: https://github.com/cjpais/Handy
- VoiceStudio — catálogo de engines: https://github.com/debpalash/VoiceStudio
- sherpa-onnx — https://github.com/k2-fsa/sherpa-onnx
- faster-whisper — https://github.com/SYSTRAN/faster-whisper
- CTranslate2 — https://github.com/OpenNMT/CTranslate2
- Opus-MT — https://github.com/Helsinki-NLP/Opus-MT
- Argos Translate — https://github.com/argosopentech/argos-translate
- MiniSBD — https://github.com/LibreTranslate/MiniSBD
- Qwen3-ASR — https://github.com/QwenLM/Qwen3-ASR
- Qwen3-TTS — https://github.com/QwenLM/Qwen3-TTS
- Chatterbox — https://github.com/resemble-ai/chatterbox
- Chatterbox pt-PT — https://huggingface.co/ResembleAI/Chatterbox-Multilingual-pt-pt
- Piper voices — https://huggingface.co/rhasspy/piper-voices
- Piper runtime — https://github.com/OHF-Voice/piper1-gpl
- espeak-ng — https://github.com/espeak-ng/espeak-ng
- Kokoro — https://huggingface.co/hexgrad/Kokoro-82M
- VoxCPM — https://huggingface.co/openbmb/VoxCPM2
- Silero VAD — https://github.com/snakers4/silero-vad
- pyannote — https://github.com/pyannote/pyannote-audio
- Apache-2.0 — https://www.apache.org/licenses/LICENSE-2.0
- MIT — https://opensource.org/license/mit
- Regulamento Europeu da IA, Artigo 50 — https://eur-lex.europa.eu/eli/reg/2024/1689/oj

## D. Registo de alterações da v12.3

### D.1 Resumo face à v12.2

| Área | v12.2 | v12.3 |
|---|---|---|
| Estrutura | Doze partes com secções 1 a 58, algumas fora de ordem e um subtítulo de interface dentro da secção errada | Treze partes, secções 1 a 82 em ordem, sem duplicações; a voz, a orquestração e a biblioteca ganham partes próprias |
| Biblioteca | Modelos descritos apenas como entradas de catálogo, sem ciclo de vida próprio | Parte VII: duas camadas, contratos, catálogo evolutivo, compatibilidade, instalar e remover, armazenamento, curadoria, novidades, importação, distribuição segura, ciclos independentes e dois modos |
| Orientações do autor | Dezasseis pontos em aberto sobre tamanho, catálogo, assinatura, Piper, Argos, corpus e jurídico | Fixados na Parte VII e nas secções 77 a 82: orçamento de 100 MB, catálogo no repositório principal, Ed25519, Piper fora do instalador, Argos fora do núcleo, corpus CC0-1.0, sem parecer jurídico nesta fase |
| Consentimento e transparência | Política de voz sem fluxo de consentimento detalhado e Artigo 50 tratado de forma genérica | Secções 5, 25, 63 e 75: ecrãs de consentimento, registo por perfil, avisos contextuais, etiqueta de síntese, proveniência no ficheiro e revogação com eliminação |
| Orquestração | Model Hub e Auto Select em cinco secções, sem ciclo de avaliação nem monitorização | Parte VI com PIMO: manifesto e níveis de evidência, filtragem, perfil de dispositivo, avaliação em três níveis, benchmarks por tarefa, seleção multicritério, composição, monitorização, fallback, ciclo de vida, cache, testes, critérios de aceitação e fases |
| Voz | Clonagem descrita dentro da estratégia de TTS | Parte II completa: política, perfil, consentimento, compatibilidade, identidade, papéis da voz neutra |
| Modelos de clonagem | CustomVoice apresentado como clonagem; Kokoro apresentado como fallback PT-PT | Base 0.6B e 1.7B são clonagem; CustomVoice usa vozes predefinidas; VoiceDesign desenha voz; Kokoro deixa de ser fallback PT-PT |
| Chatterbox | Tratado como um só candidato | PT-PT como candidato prioritário e PT-BR separado |
| Tradução | «Decisão fechada» de substituição do Argos | Estado por caminho: PT→EN medido, EN→PT por resolver, Argos em revisão |
| Validação | 3 amostras corretas citadas como indício forte | Exigido conjunto representativo e avaliação por falante nativo |
| Hardware | Regra simplista de núcleos e RAM | Perfis medidos e decisão por benchmark, memória, quantização e backend |
| Perfis de voz | Não endereçada a compatibilidade entre motores | Secção própria com camadas, derivação e eliminação da amostra |
| Privacidade | Secção genérica | Privacidade por dado, dados biométricos e política de retenção |
| IA | Referência genérica | Secção sobre o Artigo 50, com obrigações por interveniente e limites |
| Licenças | Sugestão de que a separação resolve a GPL | Fronteira técnica declarada e questão jurídica isolada |
| Roadmap | Versões nomeadas com marcos e datas implícitas | Gates e fases por dependência, sem calendário |
| Benchmarks | Dispersos | Secção única com métricas, gates e regra de publicação |
| Parlo Live | Uma entrega de reunião entre três, sem requisito de voz | Objetivo central do produto, com legendas, voz num sentido e voz bidirecional como etapas; encaminhamento de áudio por aplicação declarado não testado |

### D.2 Decisões técnicas corrigidas pela auditoria

1. Qwen3-TTS 0.6B Base e 1.7B Base são candidatos de clonagem por áudio de referência.
2. Qwen3-TTS CustomVoice não é modelo de clonagem: usa vozes predefinidas.
3. Qwen3-TTS VoiceDesign é desenho de voz, não clonagem.
4. Chatterbox Multilingual PT-PT é candidato prioritário para português europeu; PT-BR é artefacto distinto.
5. Kokoro deixa de ser fallback PT-PT sem prova de cobertura dessa variante.
6. Piper mantém-se opcional, externo e fora dos arquivos de release.
7. Nenhum modelo é declarado vencedor sem teste interno com identidade, inteligibilidade e latência.
8. Perfis de voz não são compatíveis entre motores por omissão; exigem representação derivada.
9. A tradução EN→PT fica formalmente por resolver, com OPUS-MT e M2M100 418M como candidatos.
10. A seleção por hardware deixa de usar regras simplistas e passa a depender de benchmarks.
11. A remoção do Argos do núcleo depende de um par aprovado, não de uma intenção.
12. Um scanner de licenças sem alertas deixa de ser descrito como conformidade.
13. A orquestração é determinística: regras, manifestos e medições, sem um modelo de aprendizado a escolher modelos.
14. A avaliação é progressiva em três níveis, e o nível C nunca corre na máquina do utilizador.
15. A troca de motor obedece a permanência mínima, histerese, circuit breaker e limite de tentativas.
16. Nenhum resultado de benchmark é reutilizado fora do ambiente que o produziu.
17. A aplicação e a biblioteca são camadas separadas, com ciclos de atualização distintos.
18. O instalador não incorpora pesos e o seu tamanho não depende do catálogo.
19. A compatibilidade entre aplicação, catálogo, runtime e modelo é calculada, não prometida.
20. Importar um modelo externo nunca executa código do pacote e nunca o promove a validado.
21. O instalador tem objetivo de 100 MB e limite de referência de 150 MB, sem pesos dentro.
22. O catálogo vive no repositório principal e os manifestos são assinados em Ed25519, com a chave pública na aplicação.
23. Sem certificado Windows nesta fase, publicam-se hashes e instruções, sem os apresentar como assinatura.
24. O corpus de avaliação é original, sob CC0-1.0, e cobre PT-PT, PT-BR e EN nos dois sentidos.
25. A conformidade é acompanhada numa checklist interna, revista por release, sem parecer jurídico contratado.
26. Nenhum perfil de voz existe sem declaração de origem e registo de consentimento com versão do aviso.
27. Toda a exportação de áudio sintetizado identifica a origem em metadados e na interface, mesmo sem marca de água.
28. A declaração do utilizador é registada como declaração, nunca apresentada como prova de autorização.

### D.3 Questões por resolver

- Checkpoint EN→PT redistribuível que passe o gate de fidelidade.
- Clonador aprovado para PT-PT em CPU e em GPU, com identidade medida.
- Licença dos pesos do M2M100 418M e de outros candidatos de tradução.
- Licença e redistribuição dos pesos de diarização.
- Viabilidade técnica de marcação legível por máquina em cada formato de exportação.
- Qualificação jurídica da distribuição conjunta de adaptadores copyleft.

### D.4 Verificação de coerência desta edição

- Numeração contínua de 1 a 82, sem títulos repetidos e com hierarquia de partes correta.
- A Parte VI define o orquestrador sem duplicar o Model Hub, o Auto Select ou o gestor de perfis de voz.
- A prioridade Voice Clone First aparece nas Partes I, II, IV, V, VI, XI e XIII.
- Nenhuma funcionalidade planeada é descrita como implementada; o estado está na secção 1, na secção 20 e no Apêndice E.
- Licenças, modelos e dependências usam as mesmas decisões nas Partes V, XI e XII.
- O roadmap da secção 68 não contradiz as fases da secção 69 nem os gates da secção 32.
- Os critérios de aceitação da secção 42 cobrem as garantias das secções 29 a 40.
- As decisões finais do autor aparecem nas secções 44, 46, 53, 58, 70, 77 e 80 e não contradizem o PIMO nem a biblioteca.
- O consentimento não é apenas política: a secção 5 descreve os ecrãs, o registo e a revogação, e a secção 63 a marcação por formato.
- A Parte VII não duplica o Model Hub nem o PIMO: nomeia módulos com fronteiras e diz qual decide o quê.

### D.5 Revisão de 1 de novembro de 2026: Parlo Live como objetivo central

O Live deixou de ser a última linha do roadmap para passar a ser a razão de ser da política de voz. As secções 66.1 a 66.3 fixam o ciclo bidirecional com preservação de identidade, o encaminhamento de áudio por aplicação de comunicação e as regras de uma sessão com dois perfis. O roadmap da secção 68 ganha a descrição dos três passos e o conteúdo de G-Live; a tabela de fases ganha as fases 11 a 13, sem datas. Nenhuma aplicação de comunicação é declarada compatível: todas entram como não testadas até existir medição registada, e a instalação de um driver ou extensão de áudio passa a exigir consentimento explícito.

### D.6 Revisão de 1 de novembro de 2026: identidade visual e especificação de interface

A v12.4 acrescenta a Parte XIV (secções 83 a 101) e não altera nenhum requisito funcional, de voz, de licença ou de privacidade. O que passou a estar fixado:

- Personalidade visual em cinco atributos e uma direção artística justificada, «papel e voz», com o que é proibido em cada um.
- Marca: símbolo, cinco contextos de uso e as regras que impedem versões não autorizadas do logótipo.
- Design tokens completos: paleta clara e escura com HEX, escala tipográfica, espaçamento, raios, elevação, movimento e curvas.
- Dezasseis componentes reutilizáveis com anatomia e estados, e dez estados globais com palavra, cor e comportamento.
- Arquitetura de navegação por cinco grupos, com barra lateral colapsável, atalhos e as regras que impedem ações destrutivas por navegação.
- Especificação de nove ecrãs: Home, Dictate em duas superfícies, Captions em duas, Live em quatro momentos, Voice, Meeting nos três modos, Model Hub, Definições e Onboarding.
- Onze wireframes e um mapa de ecrãs e fluxos, dentro do documento.
- Responsividade por resolução, escala de sistema e monitores, com os limites de Windows, macOS e Linux declarados.
- Acessibilidade com números: 4,5:1, 3:1, foco de 2 px, alvos de 32 e 44 px, movimento reduzido e alto contraste.
- Uma fase nova no roadmap (fase 14) com critério de conclusão próprio.

O que fica por fechar e não foi inventado nesta revisão: medição de usabilidade com pessoas reais, o comportamento do encaminhamento de áudio por aplicação de chamada, o atalho global em macOS e Linux, a inserção no campo activo, e a verificação de contraste por ferramenta automatizada em vez de cálculo manual.

### D.6.1 Presença única acrescentada à Parte XIV

Uma revisão da mesma data acrescentou a secção 102: as cinco camadas (Presença, contexto, Live, Hub, avançado), o papel do ícone na área de notificação, a regra «click first, shortcut later», a escada de estados, o que o Hub conserva e o critério de sucesso em quatro percursos. A especificação deixa de tratar a janela principal como entrada e passa a tratá-la como centro de gestão, sem retirar nenhuma capacidade a nenhum ecrã das secções 89 a 97.

### D.7 Verificação de coerência desta edição

- Numeração contínua de 1 a 102, em catorze partes, sem títulos repetidos e sem capítulos órfãos.
- A secção 102 não contradiz as secções 88 a 97: a navegação por grupos mantém-se no Hub, e nenhuma capacidade foi movida de um ecrã para outro sem o nomear.
- A Parte XIV não contradiz as Partes II, V, IX e XII: a voz, as licenças, a privacidade e a transparência mantêm a mesma redação de fundo.
- A secção 12 aponta para a Parte XIV e a Parte XIV remete para a secção 12, sem duplicar a tabela de superfícies.
- A fase 14 da secção 69 exige os ecrãs das secções 89 a 97 e os tokens da secção 85, e não cria um gate concorrente com G0, G1 ou G4.
- Nenhum ecrã da Parte XIV é descrito como implementado; o estado real está no primeiro parágrafo e na lista de alterações da v12.4.
- Os estados da secção 87 usam as mesmas palavras em toda a Parte XIV e não introduzem estados que não existam no núcleo.

### D.8 Revisão de 1 de novembro de 2026: reuniões, memória e seleção de modelos

A v12.5 acrescenta a Parte XV (secções 103 a 113) e reordena os apêndices, que passam a vir depois da Parte XIV como o índice sempre indicou. Nenhum requisito das Partes I a XIV é retirado.

- A secção 103 fixa a política «O Parlo recomenda. O utilizador decide.»: a recomendação do PIMO continua a ser o caminho normal e a escolha manual deixa de poder ser lida como proibida, com aviso obrigatório e nunca com bloqueio. As eliminações da secção 29 passam a estar descritas como técnicas, não como preferência.
- A secção 104 faz de Meetings um pilar e não uma integração do Live, e impõe reutilização de captura, encaminhamento, reconhecimento, tradução e voz, com um dono por componente.
- A secção 105 fixa falantes genéricos, nomeação só com evidência direta, nomeação retroativa com marcador original guardado e a proibição de identificar pessoas por semelhança de voz.
- A secção 106 dá ao pós-reunião uma superfície de primeira classe e distingue proposta, discussão, decisão e questão aberta, com a frase que fechou cada decisão.
- A secção 107 cria o Ask Parlo com referências obrigatórias e recusa honesta quando não há suporte no transcript.
- A secção 108 define Meeting Memory como estrutura (transcript, pessoas, decisões, tarefas, tópicos, relações) e não como colecção de resumos.
- A secção 109 põe a privacidade por dado: onde vive, o que se apaga, o que fica fora do alcance do Parlo.
- A secção 110 fixa quatro estados de suporte por plataforma e proíbe promoção sem medição.
- A secção 111 especifica a superfície durante e depois da chamada, sem opções técnicas no caminho.
- A secção 112 atualiza a hierarquia para quatro pilares, com o estado real de cada um.
- A secção 113 separa três camadas com dependências explícitas e um critério de conclusão para a primeira.

O que fica por fechar: não existe sessão de reunião implementada, nem transcrição persistida, nem diarização, nem resumo, nem memória; nenhuma plataforma de chamadas foi testada; a qualidade da nomeação de falantes não foi medida; e a pesquisa entre reuniões não tem avaliação com dados reais.

### D.9 Verificação de coerência desta edição

- Numeração contínua de 1 a 113, em quinze partes, sem títulos repetidos e sem capítulos órfãos.
- A secção 103 não contradiz a Parte VI: o PIMO continua a decidir por omissão, e a escolha manual é uma excepção explícita e reversível.
- A secção 104 não duplica o Live da secção 92 nem a secção 66: nomeia o Live como componente e mantém a captura e o encaminhamento no mesmo dono.
- A secção 105 mantém a proibição da secção 62 sobre identificação biométrica e não transforma diarização em reconhecimento de pessoas.
- As secções 106 a 108 não prometem qualidade: exigem evidência por item e recusa quando não há suporte.
- A secção 109 usa as mesmas regras da secção 61 e não introduz retenção nova.
- A secção 110 usa os mesmos estados da secção 66.2 e não declara nenhuma aplicação compatível.
- A secção 111 respeita os tokens da secção 85, os estados da secção 87 e a hierarquia de superfícies da secção 102.
- A secção 112 não retira capacidades a nenhum ecrã das secções 89 a 97: muda a ordem de importância, não a existência.
- Nenhuma capacidade desta parte é descrita como implementada.

## E. Mapa das versões

| Versão do plano | Conteúdo que fixou |
|---|---|
| v10.0 | Estado real do projeto, perfis de hardware, catálogo revisto, separação entre marcos e tags |
| v11.0 | Roadmap reordenado, Live fora do caminho crítico, licenciamento modular |
| v12.0 | Model Hub, Auto Select, decisão sobre Argos, Piper externo, memorando jurídico |
| v12.1 | Política Voice Clone First e arquitetura de perfil de voz |
| v12.2 | Hierarquia corrigida e verificação documental de Qwen3-TTS Base |
| v12.3 | Parte própria para a voz, matriz de modelos corrigida, tradução por resolver, privacidade e Artigo 50, roadmap por gates; Parte VI com o Parlo Intelligent Model Orchestrator |
| v12.3 (revisão) | Integração do PIMO: manifesto e evidência, filtragem, perfil de dispositivo, três níveis de avaliação, benchmarks por tarefa, seleção multicritério, composição, monitorização, fallback com proteção de voz, ciclo de vida, cache, testes e fases |
| v12.3 (revisão 2) | Biblioteca modular: separação aplicação e biblioteca, contratos entre módulos, catálogo evolutivo, compatibilidade entre versões, gestão por modelo, armazenamento, curadoria, avaliação de novidades, importação externa, distribuição assinada e ciclos de atualização independentes |
| v12.3 (revisão 3) | Decisões finais do autor: orçamento de 100 MB, catálogo no repositório principal com Ed25519, Piper e Argos fora do núcleo, corpus CC0-1.0, latência em P50 e P95 por perfil, checklist interna de conformidade em vez de parecer jurídico |
| v12.3 (revisão 4) | Sistema de consentimento, avisos e transparência: ecrãs antes do perfil, origem declarada, registo por perfil, sete momentos de aviso, usos proibidos, etiqueta de síntese, proveniência no ficheiro, marca de água só quando verificada e revogação com eliminação |
| v12.4 | Parte XIV: identidade visual, marca, design tokens com HEX e contrastes, dezasseis componentes, dez estados globais, navegação por cinco grupos, especificação de nove ecrãs, onze wireframes, mapa de fluxos, onboarding em cinco passos, responsividade, acessibilidade e fase 14 no roadmap |
| v12.4 (revisão) | Presença única e hierarquia de superfícies na secção 102: ícone na área de notificação, clique antes de atalho, escada de estados, Hub como centro de gestão e critério de sucesso em quatro percursos |
| v12.5 | Parte XV: seleção de modelos que recomenda sem bloquear, Meetings como pilar, falantes e nomeação retroativa, pós-reunião de primeira classe, proposta distinta de decisão, Ask Parlo com prova, Meeting Memory, privacidade da memória por dado, quatro estados de suporte por plataforma e três camadas de implementação |

Versões do produto e versões do plano são coisas diferentes. A versão do produto avança a partir da última tag publicada; a do plano avança quando este documento é revisto.

## F. Regra permanente para o agente de desenvolvimento

O agente do projeto não pede ao utilizador para comparar modelos, executar benchmarks ou escolher entre motores quando o Model Hub consegue decidir. Consulta o catálogo, seleciona o candidato aprovado e continua.

Interrompe apenas quando existir uma decisão humana genuína:

- risco de licença ou jurídico que o código não resolve;
- consentimento de voz ou autorização sobre uma amostra;
- preferência explícita do utilizador que o produto não consegue inferir;
- ausência de qualquer modelo elegível para a tarefa pedida.

Em todos os outros casos, a resposta correta é escolher, medir, registar a razão e dizer em uma frase o que foi escolhido.
