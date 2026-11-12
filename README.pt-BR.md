# Clock Study

[English](README.md)

Pomodoro, temporizador e cronômetro de mesa que funcionam offline, com um relógio flip flutuante e redimensionável.

## Baixar e instalar

Baixe o pacote do seu sistema na página de [Releases do GitHub](https://github.com/daniel-18212/clock-study/releases).

| Sistema | Pacote | Instalação |
| --- | --- | --- |
| Windows 11 x64 | `*-setup.exe` | Abra o instalador. Ele inclui o instalador offline do WebView2 para sistemas que precisam dele. |
| macOS, Apple Silicon e Intel | `*.dmg` | Abra a imagem de disco e arraste o Clock Study para Aplicativos. |
| Ubuntu / Debian x64 | `*.deb` | Abra no gerenciador de pacotes ou rode `sudo apt install ./<arquivo>.deb` na pasta do download. |
| Linux x64 compatível | `*.AppImage` | Dê permissão de execução e abra o arquivo. Algumas distribuições precisam de suporte ao FUSE. |

As prévias não têm assinatura de publicador confiável no Windows nem notarização da Apple. O Controle Inteligente de Aplicativos do Windows ou o Gatekeeper do macOS podem bloqueá-las. No Linux com Wayland, o ambiente gráfico decide se a janela flutuante fica acima de outros aplicativos.

## Usar o Clock Study

- **Pomodoro:** Defina o foco, as duas pausas e quantos blocos antecedem a pausa longa.
- **Temporizador:** Defina horas, minutos e segundos. Clique nos números para editar a duração.
- **Cronômetro:** Inicie, pause, retome, reinicie e registre voltas.
- **Janela flutuante:** Clique em **Flutuar** para compactar a janela sem interromper a contagem. Arraste ou redimensione pelos controles nativos. Clique em **Expandir** para restaurar a janela.

As durações e os tamanhos das janelas ficam salvos no dispositivo. A interface está em português do Brasil.

## Compilar e publicar

Instale Node.js 22, Rust e os [pré-requisitos do Tauri](https://v2.tauri.app/start/prerequisites/) para seu sistema.

```sh
npm install
npm run tauri dev
```

Compile no sistema de destino com `npm run tauri build -- --bundles nsis` para Windows, `--bundles dmg` para macOS ou `--bundles deb,appimage` para Linux.

O [Clock Study Desktop](.github/workflows/desktop.yml) compila nos três sistemas. Uma mudança de versão em `main` publica uma prévia com os instaladores e as somas SHA-256 depois que todos os builds passam. Atualize a versão em `package.json`, `src-tauri/Cargo.toml` e `src-tauri/tauri.conf.json` e adicione notas nos dois idiomas em `docs/releases/`. Atualize os dois READMEs juntos quando mudar a instalação, o uso ou a compilação.

## Microsoft Store

Crie um produto MSIX no [Partner Center](https://partner.microsoft.com/dashboard). Defina as variáveis do Actions `STORE_IDENTITY_NAME`, `STORE_PUBLISHER` e `STORE_PUBLISHER_DISPLAY_NAME` com os dados de **Identidade do produto**. Rode **Clock Study Desktop** com **Prepare an MSIX for Microsoft Store submission** selecionado. A Microsoft assina o MSIX após a certificação.
