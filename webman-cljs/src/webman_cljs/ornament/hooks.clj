(ns webman-cljs.ornament.hooks
  (:require [webman-cljs.ornament.styles :as styles]
            [webman-cljs.ornament.components]
            [lambdaisland.ornament :as o]
            [garden.compiler :as gc]
            [girouette.tw.preflight :as girouette-preflight]))



(defn write-css
  {:shadow.build/stage :flush}
  [build-state & args]
  (require 'webman-cljs.ornament.styles :reload)
  (styles/set-token)
  ;; Just writing out the CSS is enough, shadow will pick it up (make sure
  ;; you
  ;; have a <link href=styles.css rel=stylesheet>)
  (spit "resources/public/css/site.css"
        (str
         ;; `defined-styles` takes a :preflight? flag, but we like to have
         ;; some
         ;; style rules between the preflight and the components. This
         ;; whole bit
         ;; is optional.
         (gc/compile-css (concat girouette-preflight/preflight-v2_0_3
                                 styles/global-styles))
         "\n"
         (o/defined-styles)))
  build-state)
