(ns webman-cljs.ornament.views
  (:require [lambdaisland.ornament :as o]))

(o/defstyled loading :div
  :grid-loading-35
  [:div :c-accent]
  ([]
   [:<> [:div] [:div] [:div] [:div] [:div] [:div] [:div] [:div] [:div]]))

