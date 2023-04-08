(ns webman-cljs.subs
  (:require [re-frame.core :as rf]))

(rf/reg-sub ::search-query (fn [db _] (:search-query db)))
(rf/reg-sub ::authenticated (fn [db _] (:authenticated db)))
(rf/reg-sub ::candidates (fn [db _] (:candidates db)))
(rf/reg-sub ::cand-idx-raw (fn [db _] (:cand-idx db)))
(rf/reg-sub ::loading? (fn [db _] (:loading? db)))
(rf/reg-sub ::browser (fn [db _] (:browser db)))

(rf/reg-sub ::notification? (fn [db _] (get-in db [:notification :show?])))
(rf/reg-sub ::notification (fn [db _] (:notification db)))

(rf/reg-sub ::error? (fn [db _] (get-in db [:error :show?])))
(rf/reg-sub ::error (fn [db _] (:error db)))

(rf/reg-sub ::add-tag? (fn [db _] (get-in db [:add-tag :show?])))
(rf/reg-sub ::add-tag-init-url
  (fn [db _] (get-in db [:add-tag :init-url])))
(rf/reg-sub ::add-tag (fn [db _] (:add-tag db)))

(rf/reg-sub ::cand-idx
  :<- [::cand-idx-raw]
  :<- [::candidates]
  (fn [[idx cands] _] (mod idx (count cands))))

(rf/reg-sub ::active-url
  :<- [::cand-idx]
  :<- [::candidates]
  (fn [[idx cands] _] (:url (nth cands idx))))
