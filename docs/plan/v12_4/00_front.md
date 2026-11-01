%%C PARLO
%%S Plano Mestre Técnico, Produto e Open Source
%%M Intérprete de voz local, incremental, modular e open source
%%M VERSÃO 12.4 · 1 de novembro de 2026
%%M Documento mestre de produto, arquitetura, engenharia, qualidade, governação open source e lançamento.
%%M Os valores não medidos estão identificados como metas ou hipóteses.
%%PAGEBREAK
## Como ler este documento

Esta versão 12.4 substitui a v12.3 como fonte principal de verdade do Parlo. Mantém a arquitetura, o roadmap por capacidades, a política de modelos e o que a auditoria técnica do repositório corrigiu, e acrescenta o que faltava para o produto poder ser construído sem decisões improvisadas: a identidade visual, os ecrãs, os componentes, os estados e o onboarding, com wireframes.

A Parte XIV é a especificação de interface. As Partes I a XIII não perdem valor nem autoridade: a interface não pode contradizer um requisito de voz, de privacidade, de licença ou de latência, e quando uma regra visual colide com uma dessas partes, a parte funcional prevalece e a regra visual corrige-se.

O documento separa três coisas que a versão anterior misturava em alguns pontos: o que está implementado e medido, o que está planeado com critérios de conclusão, e o que está em avaliação sem resultado. Uma capacidade descrita neste plano não é uma capacidade existente; o estado de cada uma está na secção 1, na matriz da secção 20 e no mapa de versões do Apêndice E.

A numeração é contínua de 1 a 101, agrupada em catorze partes, sem títulos duplicados e sem capítulos órfãos. A Parte XIV, nova nesta versão, é a especificação de interface: identidade, tokens, componentes, estados, navegação, ecrã a ecrã, wireframes, onboarding, responsividade e acessibilidade. A Parte VI é a especificação do orquestrador de modelos e a Parte VII a da biblioteca modular, que mantém a aplicação leve enquanto o catálogo cresce. Cada parte pode ser lida isoladamente, mas as decisões das Partes XII, XIII e XIV prevalecem sobre descrições genéricas das partes anteriores.

## Regra de evidência

> Uma afirmação só aparece como facto quando existe uma fonte oficial consultada ou uma medição feita nesta máquina, com método e condições descritas. Tudo o resto é candidato, hipótese ou valor por medir.

- **Verificado** significa que a fonte oficial (model card, repositório, licença) foi consultada e diz aquilo que o documento afirma.
- **Medido** significa que o número saiu de uma execução instrumentada, com hardware, versões, threads e quantização indicados junto.
- **Candidato** significa que o artefacto merece teste e ainda não ganhou o direito de ser default.
- **Por medir** significa que não pode ser anunciado como capacidade do Parlo, nem usado em notas de release.
- Licença do código do motor, licença dos pesos, licença das vozes e licença dos dados são sempre avaliadas em separado; nenhuma delas se herda das outras.
- Um scanner de licenças sem alertas prova que os metadados declarados estão coerentes. Não é aprovação jurídica.

## O que muda na v12.4

- **Parte XIV nova, secções 83 a 101:** personalidade e direção artística, marca, design tokens, componentes, estados globais, arquitetura de navegação, especificação de nove ecrãs, mapa de ecrãs e fluxos, onboarding, responsividade e acessibilidade.
- **A identidade visual deixa de ser preferência:** paleta clara e escura com códigos HEX e rácios de contraste, escala tipográfica, espaçamento, raios, elevação, movimento e regras de uso do acento.
- **Direção escolhida e justificada:** papel quente, tinta e terracota — «papel e voz» —, sem vidro fosco fora das sobreposições, sem gradientes em superfícies de trabalho e sem ícones a cores.
- **Navegação por cinco grupos** (Trabalho, Voz, Reuniões, Biblioteca, Sistema) em vez de uma lista de funcionalidades soltas, com barra lateral colapsável e atalhos `Alt+1` a `Alt+5`.
- **Estados globais fixados** em dez: `idle`, `listening`, `processing`, `translating`, `speaking`, `paused`, `offline`, `warning`, `error` e `completed`, cada um com palavra, cor e comportamento iguais em todo o produto.
- **Wireframes incluídos** para Home, ilha de ditado, ecrã Dictate, Captions, Live em quatro momentos, Voice, Meeting nos três modos, Model Hub, Definições e Onboarding, além do mapa de ecrãs e fluxos.
- **Onboarding em cinco passos**, com um caminho de saída em cada um e sem conta, email ou telefone.
- **Regras por resolução, escala de sistema, vários monitores e plataforma**, com o que não está prometido declarado em vez de escondido.
- **Acessibilidade com números:** contrastes mínimos, foco visível, ordem de tabulação, leitores de ecrã, movimento reduzido e alto contraste.
- **Estado honesto do que já existe:** a janela nativa com a página local, o cartão de gravação, as legendas, a ilha de ditado com atalho global em Windows, o painel de voz e o painel de modelos estão implementados; a navegação por barra lateral, o Home, o Live expandido, o Meeting e o Model Hub como biblioteca ficam especificados e não são apresentados como existentes.

## O que muda na v12.3

- A voz passa a Parte II, com política Voice Clone First, Voice Profile, consentimento, compatibilidade entre motores e medição de identidade vocal.
- A matriz de modelos foi corrigida: Qwen3-TTS Base 0.6B e 1.7B são candidatos de clonagem; CustomVoice usa vozes predefinidas; VoiceDesign desenha voz; Chatterbox Multilingual PT-PT é candidato prioritário para português europeu e o checkpoint PT-BR é tratado à parte; Kokoro deixa de ser fallback PT-PT sem prova.
- Nenhum modelo é declarado vencedor sem teste interno. A tradução EN→PT fica explicitamente por resolver, com OPUS-MT e M2M100 418M como candidatos e critérios objetivos.
- A seleção de modelos deixa de usar regras simplistas de hardware e passa a depender de benchmarks internos, memória disponível, quantização, sistema operativo e latência.
- Acrescentam-se secções próprias para privacidade e dados biométricos, Regulamento Europeu da IA com o Artigo 50, segurança da cadeia de fornecimento e direitos de voz.
- O capítulo das licenças deixa de sugerir que a separação por processos resolve sozinha a questão GPL, e mantém Argos fora do núcleo enquanto a cadeia AGPL não estiver fechada.
- Roadmap e fases passam a ser definidos por dependências e critérios de conclusão, sem datas nem estimativas.
- A orquestração de modelos ganha especificação própria na Parte VI: PIMO, perfil de dispositivo, avaliação em três níveis, benchmarks por tarefa, seleção multicritério, composição de pipeline, monitorização com histerese, recuperação, ciclo de vida e protetor da identidade vocal.
- A biblioteca passa a ser uma camada independente na Parte VII: catálogo que evolui sem reinstalar a aplicação, compatibilidade entre versões, instalação e remoção por modelo, armazenamento com recursos partilhados, importação externa, distribuição assinada e ciclos de atualização separados.
- As decisões finais do autor ficam fixadas: objetivo de 100 MB para o instalador, catálogo no repositório principal com manifestos assinados em Ed25519, Piper e Argos fora do núcleo, corpus de avaliação original sob CC0-1.0, latência publicada em P50 e P95, e uma checklist interna de conformidade em vez de parecer jurídico.

## Índice

PARTE I · FUNDAMENTOS
1. Estado do projeto e decisões executivas
2. Visão, público e família de produtos
3. Princípios não negociáveis

PARTE II · VOZ COMO CAPACIDADE CENTRAL
4. Política Voice Clone First
5. Voice Profile, consentimento e ciclo de vida
6. Compatibilidade de perfis entre motores
7. Identidade vocal entre idiomas
8. Síntese neutra, desenho de voz e conversão de voz

PARTE III · COMPORTAMENTO E PRODUTO
9. Streaming incremental e estados de texto
10. Parlo Dictate: escrever noutra língua
11. Experiência de utilização e regra transversal
12. Design system e superfícies
13. Acessibilidade, teclado e internacionalização
14. Fluxos completos e recuperação

PARTE IV · MODELOS
15. Estratégia de modelos e regra de evidência
16. Reconhecimento de fala
17. Tradução
18. Clonagem e síntese de voz
19. Diarização, VAD e alinhamento
20. Matriz de modelos por função

PARTE V · LICENÇAS E DIREITOS
21. Licenciamento por artefacto
22. Dependências e risco copyleft
23. Pesos, vozes e datasets
24. Modelos bloqueados e catálogo experimental
25. Direitos de voz e uso aceitável
26. Conformidade automática em cada release

PARTE VI · ORQUESTRAÇÃO INTELIGENTE, MODEL HUB E AUTO SELECT
27. Arquitetura de orquestração: PIMO e fronteiras
28. Manifesto por modelo e níveis de evidência
29. Descoberta e filtragem eliminatória
30. Perfil automático do dispositivo
31. Avaliação em três níveis
32. Benchmarks com evidência por tarefa
33. Seleção multicritério
34. Composição de pipeline
35. Monitorização de runtime e adaptação
36. Fallback e proteção da identidade vocal
37. Model Hub e ciclo de vida
38. Cache de resultados e reutilização
39. Privacidade e segurança da orquestração
40. Experiência do utilizador e transparência
41. Testes automáticos da orquestração
42. Critérios de aceitação do PIMO
43. Fases de implementação do PIMO

PARTE VII · BIBLIOTECA MODULAR DE MODELOS
44. Duas camadas: aplicação e biblioteca
45. Contratos entre aplicação, biblioteca e runtimes
46. Catálogo evolutivo e descoberta de novidades
47. Compatibilidade entre versões
48. Instalar, remover e substituir
49. Otimização de armazenamento e recursos partilhados
50. Biblioteca curada e personalizada
51. Avaliação de novidades pelo PIMO
52. Importação de modelos externos
53. Distribuição segura e atualizações atómicas
54. Ciclos de atualização independentes
55. Dois modos de utilizador

PARTE VIII · ARQUITETURA E PLATAFORMA
56. Camadas, núcleo e isolamento
57. Protocolo entre núcleo e motores
58. Pipeline de áudio, latência e adaptação
59. Hardware, backends e perfis
60. Extensibilidade: CLI, API, SDK e plugins

PARTE IX · PRIVACIDADE, SEGURANÇA E LEGISLAÇÃO
61. Privacidade local-first
62. Dados pessoais e biométricos
63. Regulamento Europeu da IA e Artigo 50
64. Segurança e cadeia de fornecimento
65. Reuniões: consentimento e contexto

PARTE X · REUNIÕES
66. Parlo Meeting: Lite, Live e Chat

PARTE XI · QUALIDADE, ROADMAP E FASES
67. Lições de Handy, VoiceStudio e Wispr Flow
68. Roadmap incremental
69. Fases de execução e critérios de conclusão
70. Definição de concluída e regras de corte
71. Barra competitiva

PARTE XII · OPEN SOURCE, DISTRIBUIÇÃO E GOVERNANÇA
72. Repositório e ficheiros obrigatórios
73. Licença do código, DCO e contribuições
74. Releases, artefactos e avisos
75. Transparência na aplicação
76. Governação

PARTE XIII · DECISÕES E RISCOS
77. Decisões fechadas
78. Decisões que o Parlo toma automaticamente
79. Decisões pendentes de teste interno
80. Questões que exigem decisão humana ou validação jurídica
81. Riscos por severidade
82. Checklist final antes da primeira release

PARTE XIV · INTERFACE, IDENTIDADE VISUAL E EXPERIÊNCIA
83. Personalidade visual e direção artística
84. Marca: logótipo, símbolo e uso
85. Design tokens
86. Componentes reutilizáveis
87. Estados globais
88. Arquitetura de navegação
89. Parlo Home
90. Parlo Dictate
91. Parlo Captions
92. Parlo Live
93. Parlo Voice
94. Parlo Meeting
95. Model Hub
96. Definições
97. Onboarding
98. Mapa de ecrãs e fluxos
99. Responsividade, monitores e plataformas
100. Acessibilidade visual e de teclado
101. Regras de produto que a interface não pode violar

APÊNDICES
A. Factos por componente
B. Glossário operacional
C. Fontes primárias
D. Registo de alterações da v12.3
E. Mapa das versões
F. Regra permanente para o agente de desenvolvimento
