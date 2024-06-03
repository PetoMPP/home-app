/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./src/**/*.html', './src/**/*.css', './src/**/*.rs', './*.html'],
  theme: {
    fontFamily: {
      'sans': ['Montserrat', 'ui-sans-serif', 'system-ui', 'sans-serif'],
      'mono': ['Roboto Mono', 'ui-monospace', 'SFMono-Regular', 'monospace'],
    },
    extend: {
      height: {
        screen: ['100vh /* fallback for Opera, IE and etc. */', '100dvh'],
      }
    }
  },
  plugins: [require("daisyui")],
  daisyui: {
    themes: [
      {
        bumblebee: {
          ...require("daisyui/src/theming/themes")["bumblebee"],
        },
        coffee: {
          ...require("daisyui/src/theming/themes")["coffee"],
          "primary": "#38bdf8"
        },
      }
    ],
    darkTheme: "coffee",
  },
}

