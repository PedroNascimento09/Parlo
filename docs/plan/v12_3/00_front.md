%%C PARLO
%%S Plano Mestre Técnico, Produto e Open Source
%%M Intérprete de voz local, incremental, modular e open source
%%M VERSÃO 12.3 · 1 de novembro de 2026
%%M Documento mestre de produto, arquitetura, engenharia, qualidade, governação open source e lançamento.
%%M Os valores não medidos estão identificados como metas ou hipóteses.
%%PAGEBREAK
## Como ler este documento

Esta versão 12.3 substitui a v12.2 como fonte principal de verdade do Parlo. Mantém a arquitetura, o roadmap por capacidades e a política de modelos, e corrige o que a auditoria técnica do repositório desmentiu. A alteração estrutural é a promoção da voz a parte própria do plano: a clonagem autorizada da voz da pessoa deixa de ser um capítulo dentro da síntese e passa a ser o eixo que ordena prioridades, gates e escolhas de modelo.

O documento separa três coisas que a versão anterior misturava em alguns pontos: o que está implementado e medido, o que está planeado com critérios de conclusão, e o que está em avaliação sem resultado. Uma capacidade descrita neste plano não é uma capacidade existente; o estado de cada uma está na secção 1, na matriz da secção 20 e no mapa de versões do Apêndice E.

A numeração é contínua de 1 a 58, agrupada em doze partes, sem títulos duplicados e sem capítulos órfãos. Cada parte pode ser lida isoladamente, mas as decisões das Partes XI e XII prevalecem sobre descrições genéricas das partes anteriores.

## Regra de evidência

> Uma afirmação só aparece como facto quando existe uma fonte oficial consultada ou uma medição feita nesta máquina, com método e condições descritas. Tudo o resto é candidato, hipótese ou valor por medir.

- **Verificado** significa que a fonte oficial (model card, repositório, licença) foi consultada e diz aquilo que o documento afirma.
- **Medido** significa que o número saiu de uma execução instrumentada, com hardware, versões, threads e quantização indicados junto.
- **Candidato** significa que o artefacto merece teste e ainda não ganhou o direito de ser default.
- **Por medir** significa que não pode ser anunciado como capacidade do Parlo, nem usado em notas de release.
- Licença do código do motor, licença dos pesos, licença das vozes e licença dos dados são sempre avaliadas em separado; nenhuma delas se herda das outras.
- Um scanner de licenças sem alertas prova que os metadados declarados estão coerentes. Não é aprovação jurídica.

## O que muda na v12.3

- A voz passa a Parte II, com política Voice Clone First, Voice Profile, consentimento, compatibilidade entre motores e medição de identidade vocal.
- A matriz de modelos foi corrigida: Qwen3-TTS Base 0.6B e 1.7B são candidatos de clonagem; CustomVoice usa vozes predefinidas; VoiceDesign desenha voz; Chatterbox Multilingual PT-PT é candidato prioritário para português europeu e o checkpoint PT-BR é tratado à parte; Kokoro deixa de ser fallback PT-PT sem prova.
- Nenhum modelo é declarado vencedor sem teste interno. A tradução EN→PT fica explicitamente por resolver, com OPUS-MT e M2M100 418M como candidatos e critérios objetivos.
- A seleção de modelos deixa de usar regras simplistas de hardware e passa a depender de benchmarks internos, memória disponível, quantização, sistema operativo e latência.
- Acrescentam-se secções próprias para privacidade e dados biométricos, Regulamento Europeu da IA com o Artigo 50, segurança da cadeia de fornecimento e direitos de voz.
- O capítulo das licenças deixa de sugerir que a separação por processos resolve sozinha a questão GPL, e mantém Argos fora do núcleo enquanto a cadeia AGPL não estiver fechada.
- Roadmap e fases passam a ser definidos por dependências e critérios de conclusão, sem datas nem estimativas.

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

PARTE VI · MODEL HUB, AUTO SELECT E QUALIDADE
27. Model Hub
28. Manifesto por modelo
29. Auto Select
30. Fallback
31. Benchmarks internos e gates

PARTE VII · ARQUITETURA E PLATAFORMA
32. Camadas, núcleo e isolamento
33. Protocolo entre núcleo e motores
34. Pipeline de áudio, latência e adaptação
35. Hardware, backends e perfis
36. Extensibilidade: CLI, API, SDK e plugins

PARTE VIII · PRIVACIDADE, SEGURANÇA E LEGISLAÇÃO
37. Privacidade local-first
38. Dados pessoais e biométricos
39. Regulamento Europeu da IA e Artigo 50
40. Segurança e cadeia de fornecimento
41. Reuniões: consentimento e contexto

PARTE IX · REUNIÕES
42. Parlo Meeting: Lite, Live e Chat

PARTE X · QUALIDADE, ROADMAP E FASES
43. Lições de Handy, VoiceStudio e Wispr Flow
44. Roadmap incremental
45. Fases de execução e critérios de conclusão
46. Definição de concluída e regras de corte
47. Barra competitiva

PARTE XI · OPEN SOURCE, DISTRIBUIÇÃO E GOVERNANÇA
48. Repositório e ficheiros obrigatórios
49. Licença do código, DCO e contribuições
50. Releases, artefactos e avisos
51. Transparência na aplicação
52. Governação

PARTE XII · DECISÕES E RISCOS
53. Decisões fechadas
54. Decisões que o Parlo toma automaticamente
55. Decisões pendentes de teste interno
56. Questões que exigem decisão humana ou validação jurídica
57. Riscos por severidade
58. Checklist final antes da primeira release

APÊNDICES
A. Factos por componente
B. Glossário operacional
C. Fontes primárias
D. Registo de alterações da v12.3
E. Mapa das versões
F. Regra permanente para o agente de desenvolvimento
