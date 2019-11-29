// hack to enable css source maps
// from https://github.com/facebook/create-react-app/issues/5707#issuecomment-523654329
module.exports =
    process.env.NODE_ENV === "production"
        ? {}
        : {
              overrideWebpackConfig: ({ webpackConfig, cracoConfig, pluginOptions, context: { env, paths } }) => {
                  function traverse(obj, callback) {
                      if (Array.isArray(obj)) obj.forEach(item => traverse(item, callback))
                      else if (typeof obj === "object" && obj !== null) {
                          Object.keys(obj).forEach(key => {
                              if (obj.hasOwnProperty(key)) {
                                  callback(obj, key)
                                  traverse(obj[key], callback)
                              }
                          })
                      }
                  }

                  traverse(webpackConfig, (node, key) => {
                      if (key === "loader") {
                          if (
                              node[key].indexOf("sass-loader") !== -1 ||
                              node[key].indexOf("postcss-loader") !== -1 ||
                              node[key].indexOf("css-loader") !== -1
                          ) {
                              if (node.options) {
                                  if (node[key].indexOf("sass-loader") !== -1) {
                                      // adds /* line 88, src/app/foo.scss */ comments in sourcemaped css output
                                      node.options.outputStyle = "expanded"
                                      node.options.sourceComments = true
                                      node.options.outFile = "./css/theme.css"
                                  }
                                  node.options.sourceMap = true
                              }
                          }
                      }
                  })

                  return webpackConfig
              },
          }
