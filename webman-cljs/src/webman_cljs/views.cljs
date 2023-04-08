(ns webman-cljs.views
  (:require [re-frame.core :as rf]
            [reagent.core :as r]
            [webman-cljs.events :as events]
            [webman-cljs.db :as db]
            [webman-cljs.subs :as subs]
            [webman-cljs.ornament.components :as comp :refer
             [get-event-value icons]]
            [fork.re-frame :as fork]))

(def <sub (comp deref rf/subscribe))   ;
(def >evt rf/dispatch)

(defn error
  []
  (let [show-detail? (r/atom false)]
    (fn []
      (let [{:keys [summary detail]} (<sub [::subs/error])]
        [comp/error
         {:summary summary,
          :detail detail,
          :show-detail? show-detail?,
          :dismiss-act #(>evt [::events/dismiss-error])}]))))

(defn notification
  []
  (let [{:keys [msg type]} (<sub [::subs/notification])]
    (case type
      :success [comp/success-msg msg]
      :info [comp/info-msg msg])))

(defn open-url [url] (js/window.open url "_self"))

(defn search
  []
  [comp/search
   {:input {:on-key-down (fn [e]
                           (case (.-key e)
                             "Enter" (do (.preventDefault e)
                                         (open-url (<sub
                                                    [::subs/active-url])))
                             "ArrowUp" (do (.preventDefault e)
                                           (>evt [::events/update-cand-idx
                                                  -1]))
                             "ArrowDown"
                             (do (.preventDefault e)
                                 (>evt [::events/update-cand-idx 1]))
                             nil)),
            :value (<sub [::subs/search-query]),
            :on-change #(>evt [::events/search-query-change
                               (get-event-value %)])},
    :loading? (<sub [::subs/loading?])}])

(defn candidate-list
  []
  (let [cands (<sub [::subs/candidates])
        active-idx (<sub [::subs/cand-idx])]
    [comp/ul
     {:items (for [cand cands]
               [comp/cand-item cand
                [comp/button {:on-click #(open-url (:url cand))}
                 (:open icons)]
                [comp/button
                 {:on-click #(>evt [::events/toggle-add-tag-modal true
                                    (:url cand)])} (:tag icons)]]),
      :highlight-idx active-idx}]))

(defn misc-buttons
  []
  (comp/button-groups {:content (:sync icons),
                       :act #(>evt [::events/sync])}
                      {:content (:tag icons),
                       :act #(>evt [::events/toggle-add-tag-modal true])}))

(defn login
  []
  (fn []
    (let [key (r/atom "")]
      [comp/login
       {:on-submit (fn [e]
                     (.preventDefault e)
                     (>evt [::events/login @key])),
        :ratom key}])))

(defn tag-setter-comp
  [props]
  [comp/tag-setter
   (assoc props
          :browser (name (<sub [::subs/browser]))
          :tags (map (comp clojure.string/capitalize name) db/all-tags))])

(defn tag-setter
  []
  [fork/form
   {:form-id "tag-setter",
    :prevent-default? true,
    :initial-values {"tag" "Saved",
                     "url" (<sub [::subs/add-tag-init-url])},
    :clean-on-unmount? false,
    :on-submit #(>evt [::events/submit-tag-form %])}
   (fn [props] [comp/tag-setter
                (assoc props
                       :browser (name (<sub [::subs/browser]))
                       :tags (map (comp clojure.string/capitalize name)
                                  db/all-tags))])])


(defn with-msg
  [body]
  [:div (when (<sub [::subs/error?]) [error])
   (when (<sub [::subs/notification?]) [notification]) body])

(defn tag-setter-modal
  []
  [comp/modal
   {:title "Update Url's Tag",
    :body (with-msg [tag-setter]),
    :on-close #(>evt [::events/toggle-add-tag-modal false])}])

(defn main-panel
  []
  (into [comp/page]
        [(with-msg (if (<sub [::subs/authenticated])
                     [comp/page
                      (when (<sub [::subs/add-tag?]) [tag-setter-modal])
                      [misc-buttons] [search] [:br] [candidate-list]]
                     [login]))]))
