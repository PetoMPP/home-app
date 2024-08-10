/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./templates/**/*.html', './src/**/*.css', './src/**/*.rs'],
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
  plugins: [require("daisyui"), require('tailwindcss-animated')],
  daisyui: {
    themes: [
      {
        nord: {
          ...require("daisyui/src/theming/themes")["nord"],
        },
        business: {
          ...require("daisyui/src/theming/themes")["business"],
        },
      }
    ],
    darkTheme: "business",
  },
}

