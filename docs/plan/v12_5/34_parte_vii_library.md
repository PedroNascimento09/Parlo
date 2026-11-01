# PARTE VII · BIBLIOTECA MODULAR DE MODELOS

## 44. Duas camadas: aplicação e biblioteca

O Parlo separa o que é plataforma do que é conteúdo. A aplicação contém a experiência, a inteligência de decisão e a infraestrutura de execução; a biblioteca contém os modelos que cada utilizador escolhe instalar.

| Camada | Contém | Não contém | Cresce com o catálogo? |
|---|---|---|---|
| Aplicação | Interface, PIMO, Model Hub, Auto Select, Runtime Manager, gestor de perfis de voz, componentes essenciais de processamento, cliente de catálogo | Pesos, checkpoints, coleções de vozes, datasets | Não |
| Biblioteca | Artefactos de reconhecimento, tradução, clonagem, síntese, VAD, diarização, processamento auxiliar, idiomas e variantes | Código da aplicação | Sim, indefinidamente |
| Dados do utilizador | Perfis de voz, transcrições, traduções, histórico, preferências | Modelos | Não |

Regra de dimensionamento: o instalador não incorpora pesos e não pode crescer porque a biblioteca ganhou modelos. Uma versão nova da aplicação pode ser maior por código novo, nunca por catálogo novo.

Orçamento de tamanho: 100 MB é o objetivo do instalador sem modelos, e 150 MB o limite de referência da primeira versão. Se o objetivo for excedido, a release nomeia os componentes responsáveis pelo excesso e documenta o desvio. Segurança, estabilidade e funcionalidades essenciais não se cortam para cumprir um número.

Os dados do utilizador vivem numa terceira área, separada das duas anteriores, e sobrevivem a qualquer atualização ou remoção de modelo.

## 45. Contratos entre aplicação, biblioteca e runtimes

Os gestores nomeados na arquitetura não são processos separados nem serviços concorrentes: são módulos com fronteiras testáveis, dentro de duas peças que já existem no plano — o PIMO e o Model Hub. Criar processos autónomos para cada um duplicaria estado e criaria corridas de decisão.

| Módulo | Onde vive | Responsabilidade | Não faz |
|---|---|---|---|
| Model Manifest Manager | Model Hub | Lê, valida e versiona manifestos | Não decide seleção |
| Model Compatibility Manager | Model Hub, consultando o PIMO | Calcula compatibilidade com aplicação, runtime, SO e hardware | Não descarrega |
| Model Download Manager | Model Hub | Descarrega, retoma, verifica hash, instala atomicamente | Não avalia qualidade |
| Model Security and Integrity Manager | Model Hub | Proveniência, assinatura, hashes, proteção contra downgrade | Não escolhe modelos |
| Model Studio | Model Hub (superfície avançada) | Importa artefactos externos com verificação | Não promove a validado |
| Runtime Manager | Núcleo | Arranca, vigia e termina processos de motor | Não instala modelos |
| PIMO | Núcleo | Filtra, avalia, seleciona e monitoriza | Não escreve no catálogo |
| Auto Select | Interface sobre o PIMO | Regra de produto e explicação ao utilizador | Não tem lógica de avaliação própria |

O contrato entre camadas é explícito: a aplicação conhece o catálogo por uma versão de esquema declarada; cada modelo declara a gama de esquemas e runtimes que suporta; o Runtime Manager recusa arrancar um motor cuja versão de protocolo não esteja na gama aceite. Nenhuma camada lê ficheiros de outra por caminhos implícitos.

## 46. Catálogo evolutivo e descoberta de novidades

O catálogo é metadado, não peso. Atualiza-se de forma independente da aplicação e independente dos downloads.

| Elemento | Onde vive | Regra |
|---|---|---|
| Catálogo, manifestos e hashes | Repositório principal do Parlo, pasta `catalog/` | Versionado, assinado e independente do instalador |
| Pesos e vozes | Fontes oficiais e armazenamento próprio de artefactos | Não entram no Git nem no instalador |
| Cliente de catálogo | Aplicação | Lê um esquema versionado de uma origem fixa |
| Origem alternativa | Qualquer hospedagem estática ou CDN | Mudar de origem não exige alterar a aplicação |

1. É publicada uma versão da aplicação.
2. A biblioteca recebe modelos novos, sem tocar na aplicação.
3. O cliente atualiza o catálogo e descobre as novidades.
4. O PIMO verifica quais são compatíveis com aquele dispositivo.
5. O Model Hub apresenta as novidades relevantes e as recomendações.
6. O utilizador decide o que instalar.
7. Só os modelos escolhidos são descarregados.

Regras: descobrir não descarrega; atualizar o catálogo não substitui modelos instalados nem altera preferências; uma entrada nova entra como não instalada, mesmo que seja claramente melhor para aquele dispositivo. Se o catálogo não puder ser atualizado, a aplicação funciona com a cópia local e diz há quanto tempo não é atualizada. A consulta ao catálogo é a única chamada externa permitida e não transporta áudio, conteúdo, perfis nem identificadores de utilizador: envia a versão da aplicação e do esquema.

Os pesos nunca passam pelo repositório do Parlo. São obtidos na fonte oficial indicada no manifesto, ou num distribuidor autorizado, com licença verificada e hash confirmado. Os modelos instalados ficam numa pasta de dados do utilizador, separada dos ficheiros da aplicação e do catálogo, pelo que atualizar a aplicação não os toca.

Separar o catálogo num repositório próprio é uma opção futura, não uma decisão tomada. Só se justifica quando existir uma razão técnica concreta: catálogo grande ao ponto de tornar o repositório principal desconfortável, necessidade de publicar metadados em cadência diferente da aplicação, ou um segundo consumidor do mesmo catálogo. Até lá, um catálogo versionado no repositório principal é mais simples, auditável e não cria infraestrutura a manter.

## 47. Compatibilidade entre versões

Cada modelo declara: gama de versões de API do catálogo e do protocolo; versões de runtime compatíveis; dependências; formatos e quantizações; requisitos de hardware; sistemas operativos; funcionalidades disponíveis; limitações conhecidas; e requisitos de migração, quando existirem.

| Dimensão | Como é verificada | Consequência de falha |
|---|---|---|
| API do catálogo | Gama declarada contra a versão da aplicação | Modelo não aparece como instalável |
| Protocolo de motor | Versão negociada no arranque | Motor não arranca; erro legível |
| Runtime e bibliotecas | Presença e versão mínima | Modelo marcado incompatível |
| Formato e quantização | Assinatura do ficheiro | Não é admitido |
| Hardware | Perfil medido, não estimado | Aparece como não recomendado, com razão |
| Migração | Passos declarados | Instalação bloqueada até migrar |

A compatibilidade é calculada, não prometida. Um modelo incompatível nunca é apresentado como funcional, e um modelo descontinuado mantém-se visível com a razão e a alternativa. Introduzir modelos novos não pode degradar instalações existentes: um catálogo novo sem modelo compatível deixa a configuração atual intacta.

## 48. Instalar, remover e substituir

O utilizador pode instalar, desinstalar, substituir, manter várias versões, manter vários modelos para a mesma tarefa, filtrar por idioma, instalar alternativas, restaurar versões anteriores, libertar espaço, ver o espaço ocupado, ver dependências partilhadas e ver que modelos a configuração atual usa.

| Operação | Regra |
|---|---|
| Instalar | Espaço verificado, hash validado, instalação atómica |
| Substituir | Nova versão instalada ao lado; troca só depois de validação |
| Manter versões | Permitido, com aviso de espaço e de modelo em uso |
| Remover | Protege dependências partilhadas de outros modelos |
| Remover modelo em uso | Exige confirmação, oferece alternativa ou regresso ao automático |
| Restaurar versão | Volta à última validada, sem descarregar de novo |
| Desinstalar | Nunca apaga perfis, configurações ou dados pessoais |

A remoção de um modelo não toca em dados do utilizador. Apagar dados vocais é uma operação própria, explícita e irreversível apenas quando o utilizador confirma.

## 49. Otimização de armazenamento e recursos partilhados

| Mecanismo | Comportamento |
|---|---|
| Estimativa antes do download | Tamanho comprimido e expandido, mais margem |
| Verificação de espaço | Bloqueia antes de começar, não a meio |
| Recursos partilhados | Um ficheiro comum é guardado uma vez e referenciado por vários modelos, quando a licença o permitir |
| Deduplicação | Identificação por hash, nunca por nome de ficheiro |
| Versões antigas | Mantidas por política declarada, nunca infinitamente |
| Ficheiros temporários | Limpeza automática de downloads incompletos e cache órfã |
| Modelos sem uso | Identificados e apresentados; remoção só com autorização |
| Limpeza controlada | Pré-visualização do que sai e quanto espaço liberta |

Nunca se elimina automaticamente um modelo ou versão de que o utilizador dependa. A deduplicação só acontece quando a licença de cada artefacto o permite: dois modelos com licenças diferentes não partilham ficheiros só porque o conteúdo é igual.

## 50. Biblioteca curada e personalizada

A biblioteca não mostra tudo a todos. Organiza por categoria, função, idioma, variante, qualidade validada, hardware, tamanho, latência, compatibilidade, licença e estado de validação.

| Estado | Significado | Pode instalar? |
|---|---|---|
| Recomendado para este dispositivo | Validado e adequado ao hardware e ao idioma | Sim, com um clique |
| Validado pelo Parlo | Passou os gates, não é a melhor escolha local | Sim |
| Compatível, não totalmente validado | Deve funcionar, com margem de dúvida | Sim, com aviso |
| Experimental | Opt-in explícito, fora do modo automático | Sim, com aviso detalhado |
| Incompatível | Falta hardware, runtime, versão de API ou licença | Não; mostra o motivo |
| Descontinuado | Substituído por outro artefacto | Sim, com aviso e alternativa |

Modelos avançados continuam visíveis mesmo quando não são recomendados: um utilizador com uma workstation quer saber que existem. O que nunca acontece é apresentá-los sem a limitação ao lado.

## 51. Avaliação de novidades pelo PIMO

Quando a biblioteca ganha modelos, o PIMO compara-os com a configuração instalada em qualidade, identidade vocal, idiomas, desempenho, memória, latência, compatibilidade, estabilidade e licença.

| Situação | Comportamento |
|---|---|
| Novidade claramente melhor e compatível | Aparece como recomendação com a razão e o ganho medido |
| Novidade equivalente | Não é sugerida; evita ruído |
| Novidade melhor mas exige download grande | Recomendação com custo explícito |
| Novidade sem validação | Experimental, fora do modo automático |
| Utilizador recusou a novidade | A recusa é registada e a sugestão não volta a aparecer |

O Parlo recomenda, nunca instala nem substitui sem autorização. A configuração automática continua funcional com o que está instalado, mesmo que o utilizador ignore todas as novidades durante meses.

## 52. Importação de modelos externos

O Model Studio aceita artefactos externos quando o formato, o runtime e a arquitetura são suportados. Antes de aceitar, verifica proveniência, integridade, hash quando disponível, licença e termos, formato, dependências, compatibilidade técnica, requisitos de hardware e riscos de segurança.

| Verificação | Falha |
|---|---|
| Proveniência declarada | Importação recusada |
| Integridade e hash | Importação recusada |
| Licença identificável | Importação recusada |
| Formato e runtime suportados | Importação recusada com a lista do que falta |
| Compatibilidade de hardware | Importado como incompatível, não executável |
| Código executável dentro do pacote | Recusado por omissão |

Três categorias ficam sempre distintas na interface: importado, oficial da biblioteca e validado pelo Parlo. Um modelo importado pode ser usado, mas fica fora da seleção automática de produção até validação; e nenhum pacote corre código arbitrário só porque traz um modelo lá dentro.

## 53. Distribuição segura e atualizações atómicas

| Controle | Efeito |
|---|---|
| Manifesto assinado com Ed25519 | Autenticidade da origem, além da integridade |
| Chave pública embutida na aplicação | Verificação sem depender da rede nem de uma autoridade externa |
| Chave privada fora do repositório | Guardada pelo mantenedor, nunca versionada, com procedimento escrito |
| Rotação de chave | Duas chaves aceites durante a transição, com aviso de validade |
| Recuperação | Procedimento documentado para substituir uma chave comprometida |
| Hash SHA-256 por ficheiro | Integridade depois do download |
| Transporte seguro | Nada por canais não autenticados |
| Proteção contra downgrade | Versão mais antiga não substitui a atual sem pedido explícito |
| Retoma de download | Interrupção não obriga a recomeçar |
| Instalação atómica | Falha deixa o estado anterior funcional |
| Rollback | Atualização problemática reverte sem reinstalar tudo |
| Histórico de alterações | Cada versão de catálogo e de modelo tem registo |

A assinatura é o que distingue autenticidade de integridade: um hash confirma que o ficheiro não mudou depois de descarregado, mas não prova quem o publicou. Por isso o manifesto é assinado e a chave pública viaja dentro da aplicação.

O instalador Windows não tem assinatura Authenticode nesta fase. Publicam-se hashes SHA-256 e instruções de verificação, com o aviso honesto de que o Windows mostrará alertas de origem desconhecida. Os hashes não são apresentados como substituto da assinatura de código, e o processo de distribuição fica preparado para acrescentar Authenticode sem ser reescrito.

## 54. Ciclos de atualização independentes

| Ação | Efeito |
|---|---|
| Atualizar a aplicação | Modelos instalados continuam válidos e não são descarregados de novo |
| Atualizar a biblioteca | Aplicação não é atualizada |
| Instalar modelo novo | Nada mais muda |
| Remover modelo | Nada mais muda, exceto o espaço livre |

Uma atualização da aplicação só é obrigatória quando muda a API do catálogo, o protocolo de motor ou a interface de um componente essencial. Esses casos são declarados: o catálogo diz qual é a versão mínima da aplicação e a aplicação diz o que ganha ao atualizar. Compatibilidade retroativa é a regra; rotura é a exceção documentada.

## 55. Dois modos de utilizador

| Modo | Experiência | Quem usa |
|---|---|---|
| Automático | A aplicação identifica o dispositivo, o PIMO escolhe, o Model Hub instala o necessário, as novidades aparecem quando relevantes | Experiência completa e predefinida |
| Avançado | Biblioteca completa, instalação individual, importação, versões, preferências por idioma e tarefa, controlo de atualizações e de espaço | Quem quer decidir |

No primeiro arranque, o Parlo analisa o dispositivo e apresenta uma configuração recomendada, com o tamanho total, a finalidade de cada modelo, o espaço disponível e uma estimativa de utilidade por ligação. A instalação da configuração recomendada faz-se com uma confirmação; instalar apenas parte dela é igualmente simples. Nada é descarregado sem essa confirmação, e a biblioteca completa nunca é proposta de uma vez.

O modo avançado não é requisito de qualidade: um utilizador que nunca o abra tem a melhor configuração que o dispositivo permite. Existe para quem quer fixar escolhas, importar artefactos ou limitar downloads, e nesse caso o Parlo diz que a escolha deixou de ser automática.

