// ==UserScript==
// @name         SRS Anything LeetCode + NeetCode
// @namespace    https://srs-anything.local
// @version      0.1.0
// @description  Capture problem status changes and send to SRS backend
// @author       srs-anything
// @match        https://leetcode.com/problems/*
// @match        https://neetcode.io/problems/*
// @grant        GM_xmlhttpRequest
// @connect      localhost
// ==/UserScript==

(function () {
  "use strict";

  const API_BASE = window.localStorage.getItem("srs_api_base") || "http://localhost:3000";
  const SESSION_TOKEN = window.localStorage.getItem("srs_session_token") || "";

  const getSource = () => (window.location.hostname.includes("leetcode") ? "leetcode" : "neetcode");
  const getSlug = () => window.location.pathname.split("/").filter(Boolean).pop() || "unknown-problem";
  const getTitle = () => document.title.replace(/ - LeetCode| - NeetCode/g, "").trim();
  const getStatus = () => {
    const solvedHint = document.body.innerText.toLowerCase();
    return solvedHint.includes("accepted") || solvedHint.includes("success")
      ? "solved"
      : "unsolved";
  };

  const sendEvent = () => {
    if (!SESSION_TOKEN) {
      console.warn("[srs-anything] missing srs_session_token in localStorage");
      return;
    }

    const payload = {
      source: getSource(),
      problem_slug: getSlug(),
      title: getTitle(),
      url: window.location.href,
      status: getStatus(),
      occurred_at: new Date().toISOString(),
    };

    GM_xmlhttpRequest({
      method: "POST",
      url: `${API_BASE}/events/problem-status`,
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${SESSION_TOKEN}`,
      },
      data: JSON.stringify(payload),
      onload: (response) => {
        console.log("[srs-anything] ingested event", response.status, payload);
      },
      onerror: (error) => {
        console.error("[srs-anything] ingestion failed", error);
      },
    });
  };

  // Keep this simple for MVP: when page has been stable for a few seconds, emit one event.
  window.setTimeout(sendEvent, 4000);
})();
