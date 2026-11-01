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
| OPUS-MT ROMANCE-en | Tradução PT→EN | MIT (Opus-MT) | Apache-2.0 | Sim | Candidato |
| OPUS-MT tc-big-en-pt | Tradução EN→PT | MIT (Opus-MT) | CC-BY-4.0 | Falhou | Por resolver |
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
| Orientações do autor | Dezasseis pontos em aberto sobre tamanho, catálogo, assinatura, Piper, Argos, corpus e jurídico | Fixados na Parte VII e nas secções 77 a 82: orçamento de 100 MB, `parlo-models`, Ed25519, Piper fora do instalador, Argos fora do núcleo, corpus CC0-1.0, sem parecer jurídico nesta fase |
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
22. O catálogo vive em `parlo-models` e os manifestos são assinados em Ed25519, com a chave pública na aplicação.
23. Sem certificado Windows nesta fase, publicam-se hashes e instruções, sem os apresentar como assinatura.
24. O corpus de avaliação é original, sob CC0-1.0, e cobre PT-PT, PT-BR e EN nos dois sentidos.
25. A conformidade é acompanhada numa checklist interna, revista por release, sem parecer jurídico contratado.

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
- A Parte VII não duplica o Model Hub nem o PIMO: nomeia módulos com fronteiras e diz qual decide o quê.

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
| v12.3 (revisão 3) | Decisões finais do autor: orçamento de 100 MB, catálogo em `parlo-models` com Ed25519, Piper e Argos fora do núcleo, corpus CC0-1.0, latência em P50 e P95 por perfil, checklist interna de conformidade em vez de parecer jurídico |

Versões do produto e versões do plano são coisas diferentes. A versão do produto avança a partir da última tag publicada; a do plano avança quando este documento é revisto.

## F. Regra permanente para o agente de desenvolvimento

O agente do projeto não pede ao utilizador para comparar modelos, executar benchmarks ou escolher entre motores quando o Model Hub consegue decidir. Consulta o catálogo, seleciona o candidato aprovado e continua.

Interrompe apenas quando existir uma decisão humana genuína:

- risco de licença ou jurídico que o código não resolve;
- consentimento de voz ou autorização sobre uma amostra;
- preferência explícita do utilizador que o produto não consegue inferir;
- ausência de qualquer modelo elegível para a tarefa pedida.

Em todos os outros casos, a resposta correta é escolher, medir, registar a razão e dizer em uma frase o que foi escolhido.
