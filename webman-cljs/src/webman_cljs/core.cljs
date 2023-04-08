(ns webman-cljs.core
  (:require [reagent.dom :as rdom]
            [re-frame.core :as rf]
            [webman-cljs.events :as events]
            [webman-cljs.views :as views]
            [lambdaisland.ornament :as o]
            [webman-cljs.config :as config]))

(o/defstyled loading :div
  :grid-loading-35
  [:div :c-accent]
  ([]
   [:<> [:div] [:div] [:div] [:div] [:div] [:div] [:div] [:div] [:div]]))

(defn dev-setup [] (when config/debug? (println "dev mode")))

(defn ^:dev/after-load mount-root
  []
  (rf/clear-subscription-cache!)
  (let [root-el (.getElementById js/document "app")]
    (rdom/unmount-component-at-node root-el)
    (rdom/render [views/main-panel] root-el)))


(defn init
  []
  (rf/dispatch-sync [::events/initialize-db])
  (rf/dispatch [::events/check-authentication])
  (dev-setup)
  (mount-root)
  (rf/dispatch [::events/get-candidates]))
