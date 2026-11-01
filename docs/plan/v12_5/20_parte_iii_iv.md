# PARTE III · COMPORTAMENTO E PRODUTO

## 9. Streaming incremental e estados de texto

O Parlo começa a capturar e a processar áudio quando deteta fala. Não espera pelo fim da frase para iniciar reconhecimento, tradução ou preparação de síntese. O resultado pode ser provisório e ser corrigido antes de ficar confirmado.

### 9.1. Estados de texto

| Estado | Significado | Comportamento na interface |
|---|---|---|
| Parcial | Hipótese atual do reconhecedor; pode mudar com novo áudio | Estilo visual distinto; nunca é colado como final |
| Estável | Confirmado por consenso temporal entre hipóteses | Pode ser traduzido e preparado para emissão |
| Final | Segmento fechado por pausa, fronteira semântica ou decisão do utilizador | Persiste na transcrição e fecha a tradução |
| Corrigido | Texto provisório que mudou | Atualiza o texto e, se ainda não emitido, a tradução |
| Emissão bloqueada | Contexto insuficiente ou confiança baixa | Espera mais áudio; não inventa palavras |

### 9.2. Ciclo incremental

- Captura contínua em blocos pequenos, com timestamps monotónicos e identificador de sessão.
- Deteção de atividade de voz sem usar o silêncio como condição obrigatória para iniciar inferência.
- Reconhecimento com estado reaproveitado quando o motor o permite; reprocessamento com sobreposição quando não permite.
- Janela de contexto deslizante e região de texto provisório com estabilização por consenso entre as duas últimas hipóteses.
- Tradução apenas de segmentos suficientemente estáveis, com contexto anterior, glossário e nomes próprios.
- Síntese preparada por unidades semânticas curtas, emitida quando a probabilidade de revisão for baixa.
- Fecho do segmento por pausa, pontuação, limite de contexto ou comando, sem perder o contexto da sessão.

### 9.3. Políticas de streaming

| Política | Vantagem | Custo |
|---|---|---|
| Streaming nativo do modelo | Evita recalcular a janela toda | Depende de suporte real do motor |
| Local agreement | Confirma prefixo comum entre hipóteses | Atrasa a confirmação |
| Alinhamento por atenção | Decide com base em evidência acústica | Não é universal entre modelos |
| Janela deslizante com sobreposição | Simples e aplicável a qualquer modelo | Gasta CPU e pode repetir texto |

O modo atual do motor é reportado como reprocessamento, e não como streaming nativo, porque é isso que a implementação faz. Uma capacidade futura de streaming nativo passa a ser declarada quando existir, com medição própria.

### 9.4. Emissão de voz

Transcrever continuamente não é o mesmo que falar cada palavra reconhecida. Se a síntese emitir cedo demais, pode dizer algo que a frase seguinte desmente. O Parlo mostra texto parcial de imediato, traduz quando há contexto suficiente e usa um atraso de emissão ajustável. Em conversa, uma pequena espera inteligente é melhor do que falar depressa e dizer disparates.

## 10. Parlo Dictate: escrever noutra língua

Caso de uso que motiva a funcionalidade: «preciso de escrever um e-mail em alemão, mas sou português». O utilizador escolhe a língua de escrita, carrega no atalho e fala em português; o Parlo escreve no campo ativo o texto em alemão.

### 10.1. Dois idiomas independentes

| Conceito | Regra |
|---|---|
| Idioma falado | Definido no onboarding, é o idioma do reconhecimento; deteção automática opcional |
| Idioma de escrita | Escolhido a cada utilização, com favoritos para troca em um clique |
| Variante | PT-PT e PT-BR na origem; EN-GB, EN-US, ES-ES, ES-LatAm, DE-DE, DE-AT, DE-CH no destino, quando o motor as distinguir |
| Registo | Formal, neutro ou informal, com predefinição por língua e por aplicação |
| Estilo | Verbatim, limpo, traduzido fiel ou traduzido profissional |

### 10.2. Barra flutuante

Uma barra compacta junto à parte inferior do ecrã, sem roubar o foco da aplicação de destino, porque o overlay com foco impede a colagem no campo ativo.

```
●  A ouvir…      [ PT-PT → Deutsch ]   [ Formal ]        00:07
Original:   Boa tarde, Sr. Müller. Venho confirmar a reunião de quinta…
Escrita:    Guten Tag, Herr Müller. Ich möchte das Treffen am Donnerstag …
Enter inserir   ·   Esc cancelar   ·   Tab formal/informal   ·   Alt+L trocar língua
```

O chip mostra origem e destino. O texto parcial aparece na língua falada. Ao largar a tecla, a barra mostra que está a traduzir e depois o texto final. Rever antes de inserir é a predefinição nas primeiras utilizações e em pares ainda não validados; inserir automaticamente pode ser ativado por par de línguas.

### 10.3. Garantias de fidelidade

- Retro-tradução opcional na pré-visualização, que expõe erros grosseiros sem provar correção.
- Verificador determinístico de números, datas, valores, e-mails, URLs e nomes próprios, com alerta quando algo desaparece ou muda.
- Sinalização de negações em falta, que é o erro mais perigoso num e-mail profissional.
- Glossário e lista de «não traduzir» para nomes, marcas, siglas e termos técnicos.
- Nível de confiança por par e motor, visível na barra: validado, candidato ou experimental.
- Aviso de pivô quando a tradução passa por inglês.
- Falha nunca silenciosa: se a tradução vier vazia ou truncada, a barra diz isso e oferece inserir o original.

### 10.4. Níveis de suporte por par

| Nível | Pares | Condição para subir |
|---|---|---|
| Validado | Nenhum ainda | Avaliação humana cega, corpus de e-mails e mensagens, sessão de 30 minutos sem degradação, licença fechada |
| Candidato prioritário | PT↔EN | Modelo aprovado por par, avaliação por falante nativo, corpus próprio |
| Candidato | PT→DE, PT→ES, PT→FR e inversos | Existência de modelo direto aceitável e os mesmos critérios |
| Experimental | Qualquer par via pivô inglês | Sempre assinalado e desligado por defeito |

Nenhum par é anunciado como suportado porque um checkpoint lista o idioma. O estado de PT↔EN está descrito na secção 17 e continua dividido: PT→EN tem um caminho medido, EN→PT não tem.

## 11. Experiência de utilização e regra transversal

O Parlo privilegia a ação mais provável no contexto e esconde a complexidade até ser necessária. O utilizador não deve abrir a janela principal para ditar, traduzir, ver o estado de uma sessão ou marcar um momento. A janela principal é o centro de configuração e gestão; as ações frequentes acontecem em overlays compactos, atalhos e controlos junto ao contexto.

### 11.1. Elementos da janela principal

| Elemento | Comportamento |
|---|---|
| Estado | A ouvir, a reconhecer, a traduzir, a falar, atrasado, offline ou erro, com a razão em palavras simples |
| Voz | Estado da voz em uso: clonada autorizada, neutra ou indisponível, sempre nomeado |
| Transcrição | Texto parcial em estilo distinto, texto confirmado legível |
| Tradução | Versão provisória e confirmada, ocultável |
| Motor | Nome e versão do motor de reconhecimento, tradução e voz, e o que o Auto Select escolheu |
| Privacidade | Pausa, silenciar microfone, terminar sessão, apagar transcrição, estado de captura sempre visível |
| Fallback | Alternar para legendas ou voz neutra sem perder a sessão |

### 11.2. Primeira execução

- Explicar em linguagem simples o que é processado localmente e que permissões são necessárias.
- Perguntar o idioma falado e os idiomas de escrita favoritos.
- Detetar hardware e apresentar duas recomendações: equilibrado e mais leve.
- Mostrar o conjunto de modelos e os tamanhos antes de descarregar.
- Oferecer a criação de Voice Profile como passo opcional, com consentimento, nunca como condição de uso.
- Testar microfone, idioma e latência com uma frase de demonstração, e no Dictate ditar uma frase e vê-la traduzida.
- Guiar a configuração do microfone virtual por aplicação, sem instalar drivers sem consentimento.

### 11.3. Preferências sem sobrecarga

Preferências agrupadas por tarefa, com valores por omissão que funcionam sem serem tocados. O utilizador avançado consegue escolher motor e quantização; o utilizador normal nunca é obrigado a ver essa lista para ditar uma frase. Alterar um default do Auto Select é um override explícito, com aviso de que o Parlo deixa de escolher por si.

## 12. Design system e superfícies

Esta secção fixa as regras; a especificação completa — tokens, componentes, estados, ecrãs e wireframes — está na Parte XIV, secções 83 a 101, que prevalece sobre qualquer descrição genérica aqui presente sem a contradizer nos requisitos.

| Superfície | Função | Regra |
|---|---|---|
| Overlay de ditado | Falar e inserir texto junto ao cursor | Nunca rouba o foco; fecha com Esc |
| Barra de legendas | Ler o que está a ser dito | Tamanho, contraste e posição configuráveis |
| Janela principal | Configuração, histórico, modelos, licenças | Navegação por tarefas, sem painel técnico permanente |
| Janela de reunião | Estado, captura, pausa e transcrição | Compacta por omissão, expansível |
| Área de licenças | Transparência de componentes e modelos | Acessível em dois cliques a partir da janela principal |

Regras visuais: um só acento de cor por ecrã; estados com texto além de cor; animação apenas para indicar mudança de estado; nenhum elemento decorativo que consuma atenção durante uma chamada. A interface é em inglês, com preparação para tradução dos textos da própria aplicação.

## 13. Acessibilidade, teclado e internacionalização

- Toda a funcionalidade essencial é alcançável só com teclado, com foco visível e ordem lógica.
- Legendas e transcrições com tamanho de letra, contraste e espaçamento configuráveis, incluindo modo de alto contraste.
- Compatibilidade com leitores de ecrã nos fluxos principais: iniciar, parar, ler estado, inserir texto.
- Nenhuma informação transmitida apenas por cor ou apenas por som.
- Atalhos globais configuráveis, push-to-talk e modo mãos-livres, sem atalhos que colidam com os do sistema.
- Idioma da interface independente do idioma falado e do idioma de escrita.
- Diagnóstico de microfone que não exige enviar gravações para lado nenhum.

Acessibilidade é requisito de lançamento, não trabalho posterior: uma release que quebre a navegação por teclado nos fluxos principais não passa o gate da secção 70.

## 14. Fluxos completos e recuperação

| Fluxo | Passos | Falha prevista | Recuperação |
|---|---|---|---|
| Primeiro ditado | Instalar, escolher idioma, falar, inserir | Modelo ausente | O Parlo mostra o tamanho, descarrega e retoma |
| Ditado traduzido | Escolher destino, falar, pré-visualizar, inserir | Tradução vazia | Oferece original e nova tentativa |
| Criar Voice Profile | Gravar, validar, consentir, ouvir, ativar | Amostra ruidosa | Pede nova amostra com a razão concreta |
| Ouvir na própria voz | Escolher perfil, traduzir, sintetizar | Identidade não medida para o idioma | Avisa e oferece voz neutra identificada |
| Legendas de reunião | Ativar, transcrever, exportar | Áudio do sistema indisponível | Continua só com microfone e diz o que falta |
| Atualização | Verificar, ler notas, instalar | Falha de assinatura | Aborta e mantém a versão anterior |
| Apagar tudo | Escolher âmbito, confirmar | — | Lista o que foi apagado |

Regra transversal de recuperação: nenhuma falha de motor, modelo ou dispositivo pode perder texto, áudio já reconhecido ou um perfil consentido. O produto degrada em capacidades, nunca em dados do utilizador.

# PARTE IV · MODELOS

## 15. Estratégia de modelos e regra de evidência

Não existe um único melhor modelo para todas as etapas. O Parlo escolhe o modelo elegível por tarefa, idioma, variante, hardware, latência, qualidade e direitos de utilização. A comparação é trabalho interno; o utilizador recebe uma decisão e uma explicação curta.

O que a auditoria obriga a corrigir na versão anterior:

- Nenhum modelo é declarado vencedor por leitura de model card. Um modelo só se torna default depois de passar os gates da secção 32.
- Nenhuma regra simplista do tipo «quatro núcleos e 8 GB de RAM equivale ao modelo X» entra no produto. A seleção usa memória disponível, quantização, sistema operativo, backend, latência medida e qualidade medida.
- Candidato e recomendado são estados diferentes. Misturá-los foi o erro que colocou na matriz anterior um modelo de clonagem a passar por modelo de vozes predefinidas.

### 15.1. Regra de decisão por tarefa

| Tarefa | O que decide | O que nunca decide |
|---|---|---|
| Reconhecimento | Idioma e variante, duração da fala, memória, latência, qualidade medida | Popularidade do modelo |
| Tradução | Par de línguas, licença do checkpoint, fidelidade medida | Tamanho do modelo |
| Clonagem | Identidade vocal medida, latência, memória, cobertura do par de línguas | Rótulo «multilingual» no cartão |
| Síntese neutra | Locale, naturalidade, custo, licença da voz | Semelhança com uma pessoa |
| Diarização | Número de falantes, ruído, duração | Universalidade do modelo |

## 16. Reconhecimento de fala

| Modelo | Versão analisada | Licença do código e dos pesos | PT-PT | Papel |
|---|---|---|---|---|
| Parakeet TDT 0.6B v3 | checkpoint v3, int8 por sherpa-onnx | Pesos CC-BY-4.0; runtime Apache-2.0 | Sim, entre 25 línguas europeias | Candidato principal para reconhecimento por blocos em CPU |
| Whisper large-v3-turbo, medium, small, base | via faster-whisper e CTranslate2 | MIT no código e nos pesos; CTranslate2 MIT | Sim | Candidato de compatibilidade ampla e alta robustez |
| Qwen3-ASR 0.6B | checkpoint Apache-2.0 | Apache-2.0 | Cobertura PT na família; variante PT-PT por confirmar no checkpoint | Experimental até integração e gate de recursos |
| Nemotron 3.5 ASR 0.6B | cartão NVIDIA | Licença e runtime a verificar por versão | pt-PT indicado em fonte secundária | Pista experimental, não desktop |

O reconhecimento em produção hoje é Parakeet por omissão e Whisper como alternativa, ambos em CPU, com WER medido de 0,231 e 0,279 no corpus PT-PT de 20 frases. Qwen3-ASR permanece no catálogo como candidato experimental, com licença verificada e sem direito a default.

**Regras de seleção.** O Auto Select considera o idioma e a variante, o comprimento esperado da fala, a memória disponível, a latência medida no perfil de hardware, e a qualidade medida no corpus mais próximo. Um modelo com melhor WER médio mas latência incompatível com legendas em direto não é recomendado para esse modo.

## 17. Tradução

A tradução é a área com maior dívida técnica do projeto. A conclusão anterior de que a substituição do Argos estava resolvida não se sustenta: o caminho foi implementado e medido, mas só metade dos pares funciona.

### 17.1. Estado por caminho

| Caminho | Estado | Factos |
|---|---|---|
| Argos Translate 1.11.0 | No código publicado, em revisão | Declara MIT, mas depende de MiniSBD, que declara AGPLv3, e o código importa-a diretamente |
| OPUS-MT PT→EN (`opus-mt-ROMANCE-en`) | Medido nos pesos, não publicado | Apache-2.0; 12 de 12 frases completas, 878 ms de mediana em transformers; prefixo de idioma vaza em 5 de 12 saídas e os nomes próprios são traduzidos |
| OPUS-MT EN→PT (`opus-mt-tc-big-en-pt`) | Medido nos pesos, não publicado | CC-BY-4.0; 12 de 12 frases completas no corpus de 12, 9 de 12 com redação PT-PT correta, 1025 ms de mediana em transformers; a build CTranslate2 estava mal convertida e não termina a geração |
| OPUS-MT bilíngues `opus-mt-pt-en` e `opus-mt-en-pt` | Inacessíveis | O repositório devolveu 401 na data da medição |
| M2M100 418M | Candidato por avaliar | Código MIT; licença dos pesos por confirmar no cartão do checkpoint |
| NLLB-200 | Excluído | Pesos CC-BY-NC-4.0 |
| SeamlessM4T v2 | Excluído do catálogo normal | Pesos CC-BY-NC-4.0; 29,9 GB |

### 17.2. Decisão

- O caminho principal é NMT por par, com checkpoint aprovado individualmente e executado por CTranslate2.
- Três amostras corretas não validam um par. PT→EN só sobe a candidato prioritário com um conjunto representativo, avaliação por falante nativo e estabilidade em sessão longa.
- EN→PT continua por resolver. O critério de saída é objetivo: um checkpoint redistribuível, com licença confirmada, que produza traduções corretas num conjunto de pelo menos 100 frases com números, negações e nomes próprios, acima do limiar de fidelidade definido na secção 32.
- O Argos sai do núcleo enquanto a cadeia AGPL não tiver conclusão técnica e de licenciamento. Se voltar, volta como integração externa opcional, com a sua licença visível.
- Um modelo de linguagem grande pode existir para polimento de tom, nunca como tradutor por defeito.

## 18. Clonagem e síntese de voz

A matriz anterior confundia quatro coisas: clonagem a partir de amostra, vozes predefinidas, desenho de voz e síntese neutra. A tabela seguinte separa-as, com a licença e o estado que a auditoria confirmou.

| Checkpoint | Tamanho | Licença dos pesos | Clona voz | PT-PT | Papel |
|---|---|---|---|---|---|
| Qwen3-TTS 12Hz 0.6B Base | 2,52 GB | Apache-2.0 | Sim, a partir de áudio de referência | Cobertura PT na família; PT-PT por medir | Candidato de clonagem em CPU |
| Qwen3-TTS 12Hz 1.7B Base | 4,54 GB | Apache-2.0 | Sim, a partir de áudio de referência | Idem | Candidato de clonagem em GPU |
| Qwen3-TTS 1.7B CustomVoice | 4,52 GB | Apache-2.0 | Não: oferece vozes predefinidas | Idem | Voz neutra de qualidade, não clonagem |
| Qwen3-TTS 1.7B VoiceDesign | 4,52 GB | Apache-2.0 | Não: desenha voz por descrição | Idem | Desenho de voz, experimental |
| Chatterbox Multilingual pt-PT | 4,26 GB | MIT | Sim | Sim, checkpoint dedicado a português europeu | Candidato prioritário para clonagem PT-PT |
| Chatterbox Multilingual pt-BR | 4,26 GB | MIT | Sim | Não: variante brasileira | Catálogo separado, nunca substituto de PT-PT |
| VoxCPM2 | 4,96 GB | Apache-2.0 | Sim, zero-shot | Por medir | Experimental |
| VoxCPM 0.5B | 1,61 GB | Apache-2.0 | Sim | Por medir | Experimental |
| Kokoro 82M | 0,36 GB | Apache-2.0 | Não | Não: apenas pt-BR entre as vozes publicadas | Voz neutra de baixo custo, não fallback PT-PT |
| Piper, vozes `rhasspy/piper-voices` | 12 GB no conjunto | MIT no repositório de vozes; runtime piper1-gpl GPL-3.0-or-later | Não | Sim, voz `pt_PT-tugão-medium` | Voz neutra leve, integração externa opcional |
| F5-TTS | 1,35 GB | Código MIT, pesos CC-BY-NC | Sim | Por medir | Excluído |
| XTTS-v2 | 1,86 GB | Coqui Public Model License, não comercial | Sim | Sim | Excluído |

### 18.1. Regras derivadas

- Qwen3-TTS Base 0.6B é o candidato de clonagem para máquinas sem GPU; 1.7B Base é o candidato de qualidade em GPU. Nenhum dos dois é default antes de medir identidade vocal em português europeu.
- CustomVoice e VoiceDesign não preenchem o lugar de um clonador na cadeia da secção 4.1. Apresentá-los como clonagem era o erro corrigido nesta versão.
- Chatterbox Multilingual PT-PT é o único checkpoint com português europeu dedicado entre os candidatos verificados, e por isso recebe prioridade na avaliação, não um estatuto de vencedor.
- Kokoro continua no catálogo como voz neutra barata, para inglês e para brasileiro, e deixa de ser citado como fallback de português europeu sem prova.
- Piper permanece opcional e externo enquanto o runtime for GPL. As vozes são artefactos separados, com a licença do repositório de vozes.
- A clonagem exige consentimento da secção 5, em qualquer motor.

## 19. Diarização, VAD e alinhamento

| Componente | Licença | Estado | Papel |
|---|---|---|---|
| Silero VAD | MIT | Verificado | Deteção de atividade de voz no núcleo |
| sherpa-onnx | Apache-2.0 | Em uso | Runtime de reconhecimento e utilitários de áudio |
| Diarização sherpa-onnx | Apache-2.0 no runtime; modelos por verificar | Candidato | Diarização offline para Meeting Lite |
| pyannote | Código MIT; pesos com condições de acesso | Candidato condicionado | Diarização de referência, sujeita a confirmação de redistribuição |
| Alinhamento forçado | Depende do motor | Candidato | Timestamps por palavra para legendas e dobragem |

Regras: nenhuma diarização atribui nomes de pessoas; os falantes são identificados por número e a associação a uma pessoa é um ato manual e local do utilizador. Pesos com condições de acesso não entram em arquivos de release; são descarregados pelo utilizador com a licença à vista.

## 20. Matriz de modelos por função

Síntese de tudo o que as secções 16 a 19 verificaram. Os estados são operacionais: recomendado entra no modo automático quando os gates fecharem; compatível funciona e não é escolha automática; experimental exige opt-in; bloqueado não é distribuído; externo corre fora do núcleo.

| Função | Candidato | Estado | Motivo |
|---|---|---|---|
| Reconhecimento PT-PT por blocos | Parakeet TDT 0.6B v3 int8 | Recomendado | Medido em CPU, WER 0,231 no corpus atual |
| Reconhecimento universal | Whisper large-v3-turbo e variantes | Compatível | Robusto, MIT, cobertura ampla, mais lento |
| Reconhecimento avançado | Qwen3-ASR 0.6B | Experimental | Licença Apache-2.0 verificada; integração e recursos por medir |
| Reconhecimento em GPU NVIDIA | Nemotron 3.5 ASR | Experimental | Runtime e licença por confirmar por versão |
| Tradução PT→EN | OPUS-MT ROMANCE-en por CTranslate2 | Candidato | Medido, ainda sem corpus representativo |
| Tradução EN→PT | Nenhum aprovado | Por resolver | Todos os checkpoints testados falharam |
| Tradução por par, arquitetura | CTranslate2 com checkpoint aprovado | Recomendado | MIT, rápido, sem dependências copyleft |
| Tradução contextual | M2M100 418M | Candidato por avaliar | Licença dos pesos por confirmar |
| Clonagem em CPU | Qwen3-TTS 0.6B Base | Candidato prioritário | Apache-2.0, 2,5 GB, clonagem por referência |
| Clonagem em GPU | Qwen3-TTS 1.7B Base | Candidato | Apache-2.0, mais qualidade, mais memória |
| Clonagem PT-PT | Chatterbox Multilingual pt-PT | Candidato prioritário | MIT e checkpoint dedicado a português europeu |
| Voz neutra barata | Kokoro 82M | Compatível | Não cobre PT-PT; útil para inglês e pt-BR |
| Voz neutra externa | Piper com vozes MIT | Externo | Runtime GPL fora do núcleo |
| Desenho de voz | Qwen3-TTS VoiceDesign | Experimental | Não é clonagem; requer rotulagem |
| Diarização | sherpa-onnx ou pyannote | Candidato | Licenças e pesos por fechar |
| Deteção de atividade | Silero VAD | Recomendado | MIT, leve, em uso |
| Modelos de voz não comercial | F5-TTS, XTTS-v2, NLLB, SeamlessM4T | Bloqueado | Licenças não compatíveis com distribuição livre |
