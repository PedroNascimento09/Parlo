<p align="center">
  <img src="assets/parlo-wordmark.png" width="280" alt="Parlo">
</p>

<p align="center">
  <b>Fala para dentro. Texto para fora. No teu computador, em português e inglês.</b><br>
  Ditar, texto ao vivo, legendas, reuniões e dublagem partilham a mesma engine.<br>
  Windows primeiro, macOS e Linux a compilar. Sem conta, sem telemetria, sem envio de áudio.
</p>

<p align="center">
  <a href="https://github.com/PedroNascimento09/Parlo/releases/latest"><img alt="Última release" src="https://img.shields.io/github/v/release/PedroNascimento09/Parlo?style=flat-square&color=b03c1e"></a>
  <a href="https://github.com/PedroNascimento09/Parlo/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/PedroNascimento09/Parlo/ci.yml?style=flat-square"></a>
  <a href="LICENSE"><img alt="Licença" src="https://img.shields.io/badge/licença-Apache--2.0-1e4f4a?style=flat-square"></a>
  <a href="https://github.com/PedroNascimento09/Parlo/stargazers"><img alt="Estrelas" src="https://img.shields.io/github/stars/PedroNascimento09/Parlo?style=flat-square&color=b03c1e"></a>
</p>

O Parlo é um programa de computador que transforma a fala em escrita sem enviar nada para fora. Pressiona um atalho, fala, e a frase aparece no campo onde já estava a escrever, limpa, e traduzida se pedires outra língua. A mesma engine serve tudo: a janela serve para configurar, o atalho e a bandeija são como se usa.

## O que o Parlo faz agora

**Dita em qualquer programa.** `Ctrl+Alt+D` abre uma pequena barra em cima da tela, no monitor onde está o cursor. Fala, e a frase é escrita na janela que tinha o foco, na língua que estás a escrever. `Ctrl+Alt+Shift+D` mantém a escuta aberta enquanto precisares. Quando um programa não aceita texto escrito, a frase vai para a área de transferência, e `Escape` descarta-a.

**Lê o que dizes enquanto falas.** O texto ao vivo abre uma janela com duas colunas: o microfone à esquerda e a tradução à direita, em janelas de cerca de 2,5 segundos. As legendas colocam o mesmo fluxo numa faixa fina sobre uma chamada ou um vídeo, com um botão que muda o tamanho. `Ctrl+Alt+C` abre legendas de qualquer lugar, e `Escape` para qualquer uma delas.

**Guarda uma reunião.** Grava a sala e recebe uma transcrição com hora e falante em cada linha. Renomeias um falante uma vez e todas as linhas seguem, importas uma transcrição que já tens, pesquisas a reunião inteira, perguntas ao que foi decidido e recebes a linha, o falante e a hora por trás da resposta, e copias tudo como Markdown.

**Dublagem de um ficheiro.** Arrasta um vídeo ou áudio e recebesh o mesmo conteúdo na outra língua, com legendas sincronizadas ao original e o fundo sonoro original por baixo da nova voz, se quiseres.

**Ouve a tradução.** A frase traduzida é lida em voz alta em uma voz europeia portuguesa, na mesma janela, sem sair para outro serviço.

**Ajusta as palavras que o reconhecedor não consegue.** Um vocabulário local corrige os nomes que o reconhecedor continua a errar, palavra por palavra, antes de qualquer modelo ler a frase. O histórico guarda o que foi dito, o que foi escrito e o programa onde aí chegou, e um clique devolve uma linha à área de transferência.

## Um programa, seis sítios

A janela serve para configurar e rever. O trabalho acontece pelo atalho e pela bandeija.

- **Página inicial** diz se está pronto, qual a engine e o perfil da máquina que encontrou, e as três últimas entradas com o programa onde cada uma aí chegou.
- **Biblioteca** guarda o histórico, os modelos e o vocabulário.
- **Reuniões** guarda gravações, transcrições e falantes.
- **Voz** grava uma amostra, a selra com a tua conta Windows se pedires, reproduz e exporta marcada como sintética. Clonar uma voz a partir dessa amostra ainda não está no build.
- **Estatísticas** conta palavras, minutos e programas por trás das ditas, e as últimas quatro semanas, a partir do que fica neste computador.
- **Definições** têm cinco grupos de campos reais: Tu, Ditar, Línguas, Atalhos e Avançadas, que guarda limpeza, provedor, privacidade e sobre. Cada campo guarda onde está, e o par de línguas tem o seu próprio Aplicar.

A bandeija lista os três modos acima de um separador e os cinco destinos abaixo dele; um clique esquerdo abre a barra. A primeira vez são cinco passos: o teu nome, o par de línguas, o microfone com o local exato do Windows para o permitir, o estilo de escrita e o atalho com o que o `Escape` faz. Só o nome é obrigatório; tudo o resto pode ser ignorado, e o passo inicial não volta uma vez feito.

## Atalhos

| Teclas | O que acontece |
|---|---|
| `Ctrl+Alt+D` | Abre a barra e dita para a janela que tinha o foco |
| `Ctrl+Alt+Shift+D` | Mantém a escuta sem precisar de manter qualquer tecla |
| `Ctrl+Alt+C` | Abre ou fecha as legendas |
| `Escape` | Descarta a frase, ou para o texto ao vivo, legendas ou uma reunião em curso |
| Um clique na bandeija | A barra, a qualquer momento |

Cada tecla muda em Definições, é validada ao guardar, recusada quando se repete outra das três, e recusada com motivo quando o Windows diz que outro programa já a tem.

## O que há dentro

| | |
|---|---|
| **Um gestor de modelos que diz a verdade** | Uma linha por modelo: tamanho em disco, `Instalado · não carregado`, `Carregado`, `A usar` ou `Não instalado`, um número medido de qualidade ou o honesto “nenhum número medido”. Cada linha tem `Carregar`, `Descarregar agora` e `Remover`, e o seu atraso antes de ser libertado, de 10 minutos a 4 horas ou `Manter carregado`. Downloads podem ser parados, e a pasta de modelos toda pode mover-se para outro disco com comparação byte a byte antes de apagar a cópia antiga. |
| **Dois reconhecedores, escolhidos por máquina** | Parakeet TDT 0.6B v3 int8 é o padrão; Whisper `tiny` a `large-v3` fica selecionável aonde 640 MB é demasiado. A lista nombra os núcleos, a memória e se AVX2 está presente, e por que razão essa máquina recebeu a recomendação que recebeu. |
| **Tradução com porta** | Português e inglês, em ambas as direções. 100 frases originais são verificadas para números, relógios, negações, nomes e completude antes de uma release ser chamada boa. PT→DE corre através do inglês e é publicado como não validado por um falante nativo. |
| **Limpeza que respeita o que disseste** | Três níveis: o que o reconhecedor ouviu, a frase que querias com auto-correções e hesitações removidas, e uma versão mais arrumada com espaçamentos e maiúsculas. A frase ouvida fica sempre ao lado. |
| **Palavras que o Parlo deve aprender** | Um vocabulário local corrige os nomes que o reconhecedor continua a errar, palavra por palavra, antes de qualquer modelo ler a frase. |
| **Um histórico que podes usar de novo** | Cada dita guarda o que foi dito, o que foi escrito e o programa onde aí chegou, e um clique devolve uma linha à área de transferência. |
| **Um painel de voz com uma amostra real** | De 10 a 30 segundos são gravados na janela ou recusados com o comprimento medido; uma gravação silenciosa também é recusada; a amostra pode ser selrada como cifra DPAPI, ouvida de volta, exportada como um ficheiro marcado como sintético, ou apagada com o perfil. |
| **Um provedor só quando pedides** | O modelo na nuvem fica atrás de uma chave, a tua própria chave e uma frase de teste que prova que responde. Com a chave desligada, nada sai da máquina, e a secção de Privacidade diz isso numa frase. |
| **Atualizações dentro do app** | A última versão chega com as suas notas e um botão, e substitui a cópia em execução em vez de pedires desinstalar. Mantém `parlo.previous` ao lado do binário, e uma versão sem build para o teu sistema oferece as notas em vez de um botão morto. |
| **Uma interface feita para ler** | Cada par de textos passa 4.5:1 medido nas cores reais, os dois temas são desenhados a partir dos mesmos quinze tokens, um anel de foco atinge tudo o que aceita teclado, e nenhuma fonte, script ou imagem vem de uma CDN. |

## Instalação

**Pressa?** [Começa aqui](docs/START.md) vai de um computador limpo a uma frase dita em cinco minutos.

**Windows.** [Descarrega Parlo-Setup.exe](https://github.com/PedroNascimento09/Parlo/releases/latest/download/Parlo-Setup.exe) e dá-lhe duplo clique. Instala para o teu utilizador, adiciona uma entrada no Menu Iniciar e abre o Parlo quando termina, por isso um único descarregamento é a instalação toda. O build não está assinado, por isso o SmartScreen pede uma vez: **Mais informações**, depois **Executar mesmo assim**.

**Verifica o descarregamento.** Cada release leva um `SHA256SUMS.txt` com a hash de cada ficheiro e um registo de proveniência de build que nombra o fluxo de trabalho, o commit e o repositório que o produziu. No Windows:

```bat
certutil -hashfile Parlo-Setup.exe SHA256
```

Compara essa linha com a do instalador no `SHA256SUMS.txt`. Com o GitHub CLI autenticado, isto verifica de onde veio o ficheiro em vez de confiar na página:

```bash
gh attestation verify Parlo-Setup.exe --repo PedroNascimento09/Parlo
```

Nenhum dos dois substitui uma assinatura de código: o Windows continua a tratar o instalador como não assinado, e o SmartScreen continua a pedir uma vez.

**Gestores de pacotes.** winget, Homebrew, AUR, Snap, um pacote Debian, um AppImage e Nix têm cada um uma manifestação em [packaging](packaging/README.md) com o comando que instala. São receitas, não uma instalação testada: nenhum canal foi corrido de ponta a ponta da máquina que os escreveu, por isso cada um lá está marcado como parcial com o que ainda falta. Um Dockerfile na mesma pasta inicia o servidor de um recipiente, também parcial por nenhuma imagem ter sido construída aqui.

**Aviso do Windows.** O instalador Windows não está assinado. Pode aparecer o aviso do SmartScreen na primeira descarga; isso é esperado para um instalador novo e não assinado, e não é uma afirmação de que o ficheiro é malicioso. Podes verificar o ficheiro que descarregaste com o SHA-256 do release e, se quiseres, consultar [Realidade da assinatura no Windows](docs/WINDOWS_SIGNING_REALITY.md).

**macOS e Linux**, em qualquer terminal:

```bash
curl -fsSL https://raw.githubusercontent.com/PedroNascimento09/Parlo/main/scripts/install.sh | bash
```

**A partir do código**, com Rust 1.85 ou mais recente e Python 3.12 para a engine:

```bash
cargo build --release -p parlo-cli -p parlo-app
target/release/parlo-app
```

O instalador é leve porque separa o programa de parte pesada. O programa instala numa fracção de segundo; a engine pesada é um descarregamento à parte, e o primeiro uso dentro da app oferece o reconhecedor, cerca de 640 MB, e nada mais é descarregado até pressionares o seu botão. Tradução, leitura em voz alta e legendas são chaves separadas, e cada uma nombra o modelo e o seu tamanho antes de qualquer coisa se mover. Depois disso a máquina pode ficar desconectada. `parlo install` e `parlo uninstall` fazem o mesmo a partir de um terminal, e a desinstalação mantém os teus modelos a menos que adicionares `--purge`.

## A partir do terminal

A janela não é a única porta. A mesma engine responde na linha de comando, para scripts e para uma máquina sem ambiente gráfico:

```bash
parlo dictate                 # uma frase para a área de transferência
parlo stream in.wav           # texto parcial, estável e final à medida que chega
parlo listen --play --speaker 3   # traduz o microfone para outra saída
parlo dub talk.mp4 --srt talk.srt # um ficheiro dublado e as suasLegendas
parlo serve                   # a api local autenticada, só em loopback
parlo profile                 # o que esta máquina pode fazer, e o que carregar
parlo doctor                  # se a captura e a engine estão saudáveis
```

Cada iniciação do programa deixa um registo em `sessions.jsonl` na pasta de dados: o momento em que começou, o momento em que um fim limpo o terminou, ou a mensagem e o ficheiro e linha de um pânico, mantidos os mais recentes 400. `parlo doctor` lê esse ficheiro de volta e imprime as sessões registadas, quantas foram limpas, quantas caíram, quando a última terminou e a taxa sem queda, e diz numa linha quando ainda não tem nada registado.

## Medido

Portátil Windows 11, sem GPU, 4 threads. Condições e cada linha bruta estão em [docs/BENCHMARKS.md](docs/BENCHMARKS.md).

| O que | Resultado |
|---|---|
| Regras de limpeza, 10 frases × 1 000 correes | **61,9 µs** em média |
| Um segmento pela cascata, reconhecimento incluído | **420,7 ms** em média, 485 ms p95 |
| Fator em tempo real, Parakeet v3 int8 | **0,11** |
| Legendas, fala para ecrã numa janela de 1,2 s | **cerca de 1,6 s** |
| Fluxo, primeiro parcial a frase concluída | **849 ms** a **3 355 ms** |
| Reconhecedor, 20 frases de português europeu | **0,24** taxa de erro de palavra |
| Início a frio, engine incluída | **18,1 s** |

Nada aqui descreve uma máquina que ninguém correu, e nenhuma latência ou preço de provedor é alegado: nenhum pedido a um provedor na nuvem foi feito com uma chave real.

## Privacidade

O modo local não faz nenhuma chamada de rede. O áudio nunca é escrito em disco a menos que passares um caminho de saída. A api local escuta em loopback com um token que a janela gera em cada iniciação, e a verificação de atualizações é a única chamada externa, levando nenhum áudio e nenhum identificador. Um provedor só é atingido quando o permites, guardas a tua própria chave e escolhes, por HTTPS ou loopback, diretamente da tua máquina. Histórico, reuniões e vocabulário ficam na tua pasta de dados de utilizador e cada um pode ser apagado da interface, uma amostra de voz pode ser selrada com a tua conta Windows, e [docs/PRIVACY.md](docs/PRIVACY.md) mostra como verificar tudo isto.

## O que ainda não está aqui

- O atalho de ditação é só em Windows. Em macOS e Linux o texto vai para a área de transferência.
- Windows está validado de ponta a ponta, janela incluída. macOS e Linux são compilados e testados pelo CI, mas a janela não foi aberta ali.
- Português e inglês são o único par. PT→DE corre através do inglês e é publicado como **não validado por um falante nativo**.
- A ditação é testada, ainda não validada com gravações de pessoas diferentes do autor.
- O build não está assinado, e não há `.msi`, `.dmg` ou `.deb` assinado.
- As reuniões gravam o microfone em vez do som de outro aplicativo, ainda não há resumo de reunião, e os falantes são etiquetados pelo reconhecedor em vez de separados por um modelo treinado para isso.
- Legendas e texto ao vivo leem o microfone, não o áudio do sistema, e atingir um aplicativo de chamada ainda precisa de um cabo virtual hoje.
- A leitura em voz alta usa uma voz europeia portuguesa neutra que nenhum grupo de ouvintes julgou, e nenhuma voz é clonada a partir de uma gravação.

## Documentos

[Start here](docs/START.md) · [App guide](docs/APP.md) · [Interface](docs/UX.md) · [Architecture](docs/ARCHITECTURE.md) · [Engines](docs/ENGINES.md) · [Models](docs/MODEL_MANAGER.md) · [Model policy](docs/MODEL_POLICY.md) · [Plugins](docs/PLUGIN_DEVELOPMENT.md) · [Dubbing](docs/DUB.md) · [Benchmarks](docs/BENCHMARKS.md) · [Install](docs/INSTALL.md) · [Privacy](docs/PRIVACY.md) · [Troubleshooting](docs/TROUBLESHOOTING.md) · [Changelog](CHANGELOG.md) · [Contributing](CONTRIBUTING.md)

## Licença

O código é **Apache-2.0**. Os pesos dos modelos mantêm os termos dos respetivos editores: Parakeet é CC-BY-4.0, os pacotes OPUS-MT são CC BY 4.0, e o adaptador Piper opcional é GPL-3.0-or-later, instalado separadamente e nunca embarcado num arquivo. [LICENSES.md](LICENSES.md) é o índice e [NOTICES_DE_TERCEIROS.md](THIRD_PARTY_NOTICES.md) carrega os avisos.
