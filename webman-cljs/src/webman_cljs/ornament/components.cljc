(ns webman-cljs.ornament.components
  (:require [lambdaisland.ornament :as o]
            #?(:cljs ["react-icons/io5" :refer
                      [IoAlertCircleOutline IoSync]])
            #?(:cljs ["react-icons/bi" :refer
                      [BiSearch BiWindowOpen BiPurchaseTag BiSync]])
            #?(:cljs ["react-icons/gr" :refer [GrClose]])
            #?(:cljs ["react-icons/md" :refer
                      [MdError MdRemoveRedEye MdOutlineSync]])))

(def icons
  #?(:clj {}
     :cljs {:error [MdError],
            :eye [MdRemoveRedEye],
            :alert [IoAlertCircleOutline],
            :close [GrClose],
            :search [BiSearch],
            :sync [MdOutlineSync],
            :tag [BiPurchaseTag],
            :open [BiWindowOpen]}))

(defn get-event-value
  [e]
  (-> e
      .-target
      .-value))

(o/defstyled loading :div
  :grid-loading-35
  [:div :c-accent]
  ([]
   [:<> [:div] [:div] [:div] [:div] [:div] [:div] [:div] [:div] [:div]]))

(o/defstyled error :div
  :p-4
  :mb-4
  :text-red-800
  :border
  :border-red-300
  :rounded-lg
  :bg-red-50
  :dark:bg-gray-800
  :dark:text-red-400
  :dark:border-red-800
  [:hr :border-red-300]
  [:h3 :text-lg :font-medium]
  [:.content :mt-2 :mb-4 :text-sm]
  [:.button-container :flex]
  [:.more-button :text-white :bg-red-800 :hover:bg-red-900 :focus:ring-4
   :focus:outline-none :focus:ring-red-300 :font-medium :rounded-lg
   :text-xs :px-3 :py-1.5 :mr-2 :text-center :inline-flex :items-center
   :dark:bg-red-600 :dark:hover:bg-red-700 :dark:focus:ring-red-800]
  [:.dismiss-button :text-red-800 :bg-transparent :border :border-red-800
   :hover:bg-red-900 :hover:text-white :focus:ring-4 :focus:outline-none
   :focus:ring-red-300 :font-medium :rounded-lg :text-xs :px-3 :py-1.5
   :text-center :dark:hover:bg-red-600 :dark:border-red-600
   :dark:text-red-500 :dark:hover:text-white :dark:focus:ring-red-800]
  [:.icon :pr-2 :text-lg]
  [:>.container :flex :items-center]
  ([{:keys [summary detail show-detail? dismiss-act]}]
   [:<> [:div.container [:div.icon (:error icons)] [:h3 "Error!"]]
    [:div.content summary]
    (when @show-detail? [:div [:hr] [:div.content detail]])
    [:div.button-container
     (when (not @show-detail?)
       [:button.more-button
        {:type "button", :on-click #(reset! show-detail? true)}
        [:div.icon (:eye icons)] "View more"])
     [:button.dismiss-button {:type "button", :on-click dismiss-act}
      "Dismiss"]]]))

(o/defstyled message :div
  :flex :items-center
  :text-white [:p :text-sm :font-bold]
  :px-4 [:>.icon :pr-2 :mt-0.8]
  :py-3 ([icon msg] [:<> [:div.icon icon] [:p msg]]))

(o/defstyled info-msg :div
  [message :bg-blue-500]
  ([msg] [message (:alert icons) msg]))

(o/defstyled success-msg :div
  [message :bg-green-500]
  ([msg] [message (:alert icons) msg]))

(o/defstyled page :div
  :flex :flex-col
  :items-stretch :justify-start
  :c-bg ([& c] (into [:<>] c)))

(o/defstyled text :p :text-sm :font-medium :c-text :truncate)
(o/defstyled secondary-text :p
  :text-sm :font-medium
  :truncate :c-secondary-text)
(o/defstyled button :button
  :items-center
  :px-2
  :py-2
  text
  [:&:hover :c-accent-text])

(o/defstyled text-button :button
  :items-center
  :px-2 :py-2
  :c-accent [:&:hover :shadow-xl :underline])

(o/defstyled button-groups :div
  [button :text-lg :px-1]
  ([& buttons]
   (into [:<>]
         (for [{:keys [content act]} buttons]
           [button {:on-click act} content]))))


(o/defstyled input :input
  :c-border
  :p-4 :c-text
  :c-shadow :focus:outline-none)

(o/defstyled search :div
  :relative
  [:>.icon :absolute :inset-y-0 :left-0 :flex :items-center :pl-3
   :pointer-events-none :c-text]
  [:input input :w-full :border-b-2 :c-border :pl-12]
  ([{:keys [input loading?]}]
   [:<> [:div.icon (if loading? [loading] (:search icons))]
    [:input#search-input
     (merge {:auto-focus true, :auto-complete "off", :type "text"}
            input)]]))

(o/defstyled login :div
  :flex
  :c-bg
  :h-screen
  [:>.login-box :border :c-border :rounded-lg :shadow-xl :p-6 :m-auto :flex
   :flex-col :flex-wrap :justify-between]
  [:input input :h-2 :border :c-border :mt-6 :mr-5]
  ([{:keys [on-submit ratom]}]
   [:div.login-box [:h2 "Input Api Key to continue..."]
    [:div.input-div
     [:form {:on-submit on-submit} [:label {:for "password"} "Api Key: "]
      [:input
       (merge {:auto-focus true,
               :type "password",
               :name "password",
               :value @ratom,
               :on-change #(reset! ratom (get-event-value %))})]
      [text-button {:type "submit"} "Login"]]]]))

(o/defstyled ul :ul
  :divide-y
  :c-border
  [:>.item :pb-1 :sm:pb-2 :pl-5]
  [:>.selected :c-base]
  ([{:keys [items highlight-idx], :or {highlight-idx 0}}]
   (into [:<>]
         (map-indexed (fn [idx item] [:li.item
                                      (when (= idx highlight-idx)
                                        {:class ["selected"]}) item])
                      items))))

(o/defstyled cand-item :div
  :flex
  :items-center
  :space-x-4
  [:>.buttons :inline-flex :rounded-md :shadow-sm :hidden]
  [:&:hover [:>.buttons :inline-flex]]
  [:>.text-div :flex-1 :min-w-0]
  ([{:keys [url title tag privacy]} & buttons]
   [:<> [:div.text-div [text title] [secondary-text url]]
    (into [:div.buttons] buttons)]))

(o/defstyled modal :div
  :fixed
  :top-10
  :m-auto
  :z-50
  :w-full
  :p-4
  [:h3 :text-xl :font-semibold :c-text]
  [button :text-lg :p-1.5 :ml-auto :inline-flex :items-center]
  [:.content :relative :w-full :h-full
   {:box-shadow
    "rgba(0, 0, 0, 0.02) 0px 1px 3px 0px, rgba(27, 31, 35, 0.15) 0px 0px 0px 1px;"}]
  [:.container :relative :c-shadow
   [:>.header :flex :items-start :justify-between :p-4]]
  [:.body :p-6 :space-y-6]
  ([{:keys [title body on-close]}]
   [:<> {:tab-index "-1"}
    [:div.content
     [:div.container
      [:div.header [:h3 title]
       [button {:on-click on-close} (:close icons)]] [:div.body body]]]]))


(o/defstyled radio :div
  :float-left
  [:label :float-left :block :pr-8 :c-text]
  [:input :float-left :mr-1 :mt-1]
  ([label props]
   [:<> [:input (merge {:type "radio", :id label, :value label} props)]
    [:label {:for label} label]]))

(o/defstyled radio-groups :fieldset
  ([{:keys [values handle-change handle-blur labels name]}]
   (into [:<>]
         (for [label labels]
           [radio label
            {:name name,
             :checked (= label (values name)),
             :on-change handle-change,
             :on-blur handle-blur}]))))

(o/defstyled tag-setter :form
  :flex
  :flex-col
  :justify-between
  :items-stretch
  :justify-items-end
  :flex-nowrap
  {:min-height "200px", :max-height "80vh"}
  [:label :c-text]
  [:.url-input :border :c-border]
  [:.tag-input :float-left]
  [text :whitespace-normal]
  [text-button :items-end]
  ([{:keys [values form-id handle-change handle-blur submitting?
            handle-submit browser tags],
     :as props}]
   [:<> {:id form-id, :on-submit handle-submit}
    [:label {:for "url"} "Url: "]
    [:input.url-input
     {:auto-focus true,
      :name "url",
      :type "url",
      :value (values "url"),
      :on-change handle-change,
      :on-blur handle-blur}]
    [text
     [:i "Url will be automatically added with browser " [:u browser]
      " if it does not exist"]]
    [radio-groups (assoc props :name "tag" :labels tags)]
    [text-button {:type "submit", :disabled submitting?} "update"]]))
