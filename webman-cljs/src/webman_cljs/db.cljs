(ns webman-cljs.db
  (:require [cljs.reader]
            [cljs.spec.alpha :as s]
            [re-frame.core :as re-frame]
            ["react-device-detect" :refer
             [isChrome isChromium isFirefox isSafari]]
            [malli.core :as m]
            [malli.transform :as mt]
            [malli.error :as me]))


(def all-tags [:normal :saved :favorite :readlater])
(def all-browsers [:chromium :firefox :safari :unknown])

(def ^:private Query (m/schema [:string {:min 1}]))
(def ^:private Url (m/schema [:string {:min 1}]))
(def ^:private Tag (m/schema (into [:enum] all-tags)))
(def ^:private Privacy (m/schema [:enum :normal :private]))
(def ^:private Candidate
  (m/schema [:map [:url Url] [:title :string] [:tag Tag]
             [:privacy Privacy]]))
(def ^:private Browser (m/schema (into [:enum] all-browsers)))
(def ^:private Candidates (m/schema [:vector Candidate]))

(def ^:private Db
  (m/schema [:map [:search-query :string] [:authenticated :boolean]
             [:candidates Candidates] [:cand-idx :int] [:loading? :boolean]
             [:add-tag [:map [:show? :boolean] [:init-url :string]]]
             [:notification
              [:map [:msg :string] [:type [:enum :success :info]]
               [:show? :boolean]]] [:browser Browser]
             [:error
              [:map [:summary :string] [:detail :string]
               [:show? :boolean]]]]))

(defn- coercer
  [schema transformer]
  (m/coercer schema
             transformer
             identity
             (fn [err]
               (let [message (-> err
                                 :explain
                                 me/humanize)
                     value (:value err)]
                 (throw (ex-info (str "Invalid DB Schema: " message
                                      ", value: " value)
                                 {}))))))

(def ->candidates (m/decoder Candidates mt/json-transformer))
(def conform-db (coercer Db nil))
(def conform-query (m/coercer Query nil identity (constantly ",n")))

(defn detect-browser
  []
  (cond (or isChromium isChrome) :chromium
        isFirefox :firefox
        isSafari :safari
        :else :unknown))

(def default-db
  {:search-query "",
   :authenticated true,
   :candidates [],
   :cand-idx 0,
   :loading? true,
   :add-tag {:show? false, :init-url ""},
   :notification {:msg "", :type :info, :show? false},
   :browser (detect-browser),
   :error {:summary "", :detail "", :show? false}})


