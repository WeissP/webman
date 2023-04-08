(ns webman-cljs.events
  (:require [re-frame.core :as rf]
            [reagent.core :as r]
            [webman-cljs.db :as db]
            [ajax.core :as ajax]
            [day8.re-frame.http-fx]
            [fork.re-frame :as fork]
            [re-frame-fx.dispatch]
            [clojure.string :as cs]
            [day8.re-frame.tracing :refer-macros [fn-traced]]
            [day8.re-frame.async-flow-fx :as async-flow-fx]))

(def check-db-interceptor (rf/after db/conform-db))

(defn base-url+ [& endpoints] (str "/" (cs/join "/" endpoints)))
(defn api+ [& endpoints] (apply base-url+ "api" endpoints))
(defn auth+ [& endpoints] (apply base-url+ "auth" endpoints))

(rf/reg-fx ::focus-to-search
           (fn []
             (r/after-render #(some-> js/document
                                      (.getElementById "search-input")
                                      .focus))))

(rf/reg-event-db ::update-authentication
  (fn [db [_ authenticated?]] (assoc db :authenticated authenticated?)))

(rf/reg-event-fx ::check-authentication
  (fn [_ _]
    {:http-xhrio {:method :get,
                  :uri (auth+ "ping"),
                  :timeout 500,
                  :response-format (ajax/text-response-format),
                  :on-success [::update-authentication true],
                  :on-failure [::update-authentication false]}}))

(rf/reg-event-fx ::login
  (fn [_ [_ api-key]]
    {:http-xhrio {:method :post,
                  :uri (auth+ "login"),
                  :params {"api_key" api-key},
                  :timeout 500,
                  :format (ajax/json-request-format),
                  :response-format (ajax/text-response-format),
                  :on-success [::check-authentication],
                  :on-failure [::alert-error "Login Failed"]}}))

(rf/reg-event-fx ::sync
  (fn [_ _]
    {:http-xhrio {:method :get,
                  :uri (api+ "sync"),
                  :timeout 5000,
                  :response-format (ajax/text-response-format),
                  :on-success [::notification "successfully synchronized"
                               :success],
                  :on-failure [::alert-error "Synchronization Failed"]}}))

(rf/reg-event-fx ::url-exists-failed
  (fn [_ [_ res]]
    {:dispatch [::alert-error "could not check whether url exists" res]}))

(rf/reg-event-fx ::url-exists (fn [_ _] {}))
(rf/reg-event-fx ::url-not-exists (fn [_ _] {}))

(rf/reg-event-fx ::url-exists-succeed
  (fn [_ [_ res]]
    {:dispatch (if (= (count res) 1)
                 [::url-exists (first res)]
                 [::url-not-exists])}))

(rf/reg-event-fx ::url-exists?
  (fn [_ [_ url]]
    {:http-xhrio {:method :get,
                  :uri (api+ "urls" "search"),
                  :url-params {:query (str ",url " url), :limit 1},
                  :timeout 800,
                  :response-format (ajax/json-response-format {:keywords?
                                                               true}),
                  :on-success [::url-exists-succeed],
                  :on-failure [::url-exists-failed]}}))

(rf/reg-event-fx ::insert-url-failed
  (fn [_ [_ res]] {:dispatch [::alert-error "could not insert url" res]}))
(rf/reg-event-fx ::insert-url-succeed (fn [_ _] {}))

(rf/reg-event-fx ::insert-url
  (fn [{:keys [db]} [_ url]]
    {:http-xhrio {:method :post,
                  :uri (api+ "urls" "insert_fake"),
                  :params {:url url, :browser (:browser db)},
                  :timeout 500,
                  :format (ajax/json-request-format),
                  :response-format (ajax/text-response-format),
                  :on-success [::insert-url-succeed],
                  :on-failure [::insert-url-failed]}}))

(rf/reg-event-fx ::update-tag-failed
  (fn [_ [_ res]] {:dispatch [::alert-error "" res]}))
(rf/reg-event-fx ::update-tag-succeed (fn [_ _] {}))

(rf/reg-event-fx ::update-tag
  (fn [_ [_ tag url]]
    (let [msg (str "set url " url " to " tag)]
      {:http-xhrio {:method :post,
                    :uri (api+ "urls" "tag"),
                    :params {:tag tag, :urls [url]},
                    :timeout 500,
                    :format (ajax/json-request-format),
                    :response-format (ajax/text-response-format),
                    :on-success [::update-tag-succeed],
                    :on-failure [::update-tag-failed]}})))

(rf/reg-event-db ::toggle-add-tag-modal
  [check-db-interceptor]
  (fn [db [_ show? & [init-url]]]
    (assoc db :add-tag {:show? show?, :init-url (or init-url "")})))

(rf/reg-event-fx ::initialize-db
  [check-db-interceptor]
  (fn-traced [] {:db db/default-db}))

(rf/reg-event-fx ::search-query-change
  (fn [{:keys [db]} [_ query]]
    {:db (assoc db :search-query query :loading? true),
     :fx [[:dispatch-debounce
           [{:id ::query-change-debounce,
             :timeout 300,
             :action :dispatch,
             :event [::get-candidates query]}]]]}))

(rf/reg-event-fx ::get-candidates
  (fn [_ [_ query]]
    {:fx [[::focus-to-search]
          [:http-xhrio
           {:method :get,
            :uri (api+ "urls" "search"),
            :timeout 8000,
            :url-params {:limit 20, :query (db/conform-query query)},
            :response-format (ajax/json-response-format {:keywords? true}),
            :on-success [::get-candidates-succeed],
            :on-failure [::alert-error "Could not get results"]}]]}))

(rf/reg-event-db ::get-candidates-succeed
  [check-db-interceptor]
  (fn [db [_ result]]
    (assoc db
           :candidates (db/->candidates result)
           :cand-idx 0
           :loading? false)))

(rf/reg-event-db ::alert-error
  [check-db-interceptor]
  (fn [db [_ summary result]]
    (assoc db
           :error
           {:summary summary, :detail (str result), :show? true})))

(rf/reg-event-db ::wrap-error
  [check-db-interceptor]
  (fn [db
       [_
        {:keys [summary detail], :or {summary identity, detail identity}}]]
    (-> db
        (update-in [:error :summary] summary)
        (update-in [:error :detail] detail))))

(rf/reg-event-db ::dismiss-error
  [check-db-interceptor]
  (fn [db _] (assoc-in db [:error :show?] false)))

(rf/reg-event-fx ::notification
  [check-db-interceptor]
  (fn [{:keys [db]} [_ msg type]]
    {:db (assoc db :notification {:msg msg, :type type, :show? true}),
     :fx [[:dispatch-later
           {:ms 2000, :dispatch [::dismiss-notification]}]]}))

(rf/reg-event-db ::dismiss-notification
  [check-db-interceptor]
  (fn [db _] (assoc-in db [:notification :show?] false)))

(rf/reg-event-db ::update-cand-idx
  (fn [db [_ delta]] (assoc db :cand-idx (+ (:cand-idx db) delta))))

(rf/reg-event-fx ::submit-tag-form
  (fn [_ [_ {:keys [values dirty path]}]]
    (let [url (get values "url")
          tag (clojure.string/lower-case (get values "tag"))]
      {:async-flow {:first-dispatch [::url-exists? url],
                    :rules [{:when :seen-any-of?,
                             :events [::url-exists ::insert-url-succeed],
                             :dispatch [::update-tag tag url]}
                            {:when :seen?,
                             :events ::url-not-exists,
                             :dispatch [::insert-url url]}
                            {:when :seen?,
                             :events ::update-tag-succeed,
                             :dispatch-n [[::toggle-add-tag-modal false]
                                          [::notification
                                           (str "Successfully set " url
                                                " as " tag) :success]]}
                            {:when :seen-any-of?,
                             :events [::url-exists-failed
                                      ::insert-url-failed
                                      ::update-tag-failed],
                             :dispatch [::wrap-error
                                        {:summary
                                         (fn [old]
                                           (let
                                             [msg
                                              (str
                                               "Could not update tag of "
                                               url
                                               "\n\n")]
                                             (if
                                               (clojure.string/starts-with?
                                                old
                                                msg)
                                               old
                                               (str msg old))))}]}]}})))
