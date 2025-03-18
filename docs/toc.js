// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="start.html"><strong aria-hidden="true">1.</strong> Start</a></li><li class="chapter-item expanded "><a href="account.html"><strong aria-hidden="true">2.</strong> Account</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="account_balance.html"><strong aria-hidden="true">2.1.</strong> Balance</a></li><li class="chapter-item expanded "><a href="account_creation.html"><strong aria-hidden="true">2.2.</strong> Creation</a></li><li class="chapter-item expanded "><a href="account_nonce.html"><strong aria-hidden="true">2.3.</strong> Nonce</a></li></ol></li><li class="chapter-item expanded "><a href="data_submission.html"><strong aria-hidden="true">3.</strong> Data Submission</a></li><li class="chapter-item expanded "><a href="batch.html"><strong aria-hidden="true">4.</strong> Batch</a></li><li class="chapter-item expanded "><a href="block.html"><strong aria-hidden="true">5.</strong> Block</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="block_transaction_all.html"><strong aria-hidden="true">5.1.</strong> All Transactions</a></li><li class="chapter-item expanded "><a href="block_transaction_all_static.html"><strong aria-hidden="true">5.2.</strong> All Transactions Static</a></li><li class="chapter-item expanded "><a href="block_transaction_by_app_id.html"><strong aria-hidden="true">5.3.</strong> Transactions Filtered By App Id</a></li><li class="chapter-item expanded "><a href="block_transaction_by_app_id_static.html"><strong aria-hidden="true">5.4.</strong> Transactions Filtered By App Id Static</a></li><li class="chapter-item expanded "><a href="block_transaction_by_hash.html"><strong aria-hidden="true">5.5.</strong> Transactions Filtered By Tx Hash</a></li><li class="chapter-item expanded "><a href="block_transaction_by_hash_static.html"><strong aria-hidden="true">5.6.</strong> Transactions Filtered By Tx Hash Static</a></li><li class="chapter-item expanded "><a href="block_transaction_by_index.html"><strong aria-hidden="true">5.7.</strong> Transactions Filtered By Tx Index</a></li><li class="chapter-item expanded "><a href="block_transaction_by_index_static.html"><strong aria-hidden="true">5.8.</strong> Transactions Filtered By Tx Index Static</a></li><li class="chapter-item expanded "><a href="block_transaction_by_signer.html"><strong aria-hidden="true">5.9.</strong> Transactions Filtered By Signer</a></li><li class="chapter-item expanded "><a href="block_transaction_by_signer_static.html"><strong aria-hidden="true">5.10.</strong> Transactions Filtered By Signer Static</a></li><li class="chapter-item expanded "><a href="block_data_submission_all.html"><strong aria-hidden="true">5.11.</strong> All Data Submissions</a></li><li class="chapter-item expanded "><a href="block_data_submission_by_app_id.html"><strong aria-hidden="true">5.12.</strong> Data Submissions Filtered By App Id</a></li><li class="chapter-item expanded "><a href="block_data_submission_by_hash.html"><strong aria-hidden="true">5.13.</strong> Data Submissions Filtered By Hash</a></li><li class="chapter-item expanded "><a href="block_data_submission_by_index.html"><strong aria-hidden="true">5.14.</strong> Data Submissions Filtered By Index</a></li><li class="chapter-item expanded "><a href="block_data_submission_by_signer.html"><strong aria-hidden="true">5.15.</strong> Data Submissions Filtered By Signer</a></li><li class="chapter-item expanded "><a href="block_events.html"><strong aria-hidden="true">5.16.</strong> Events</a></li></ol></li><li class="chapter-item expanded "><a href="transaction.html"><strong aria-hidden="true">6.</strong> Transaction</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="transaction_execute.html"><strong aria-hidden="true">6.1.</strong> Execute</a></li><li class="chapter-item expanded "><a href="transaction_execute_and_watch_inclusion.html"><strong aria-hidden="true">6.2.</strong> Execute And Watch Inclusion</a></li><li class="chapter-item expanded "><a href="transaction_execute_and_watch_finalization.html"><strong aria-hidden="true">6.3.</strong> Execute And Watch Finalization</a></li><li class="chapter-item expanded "><a href="transaction_options.html"><strong aria-hidden="true">6.4.</strong> Options</a></li><li class="chapter-item expanded "><a href="transaction_payment.html"><strong aria-hidden="true">6.5.</strong> Payment</a></li></ol></li><li class="chapter-item expanded "><a href="proxy.html"><strong aria-hidden="true">7.</strong> Proxy</a></li><li class="chapter-item expanded "><a href="custom_connection.html"><strong aria-hidden="true">8.</strong> Custom Client</a></li><li class="chapter-item expanded "><a href="http_connection.html"><strong aria-hidden="true">9.</strong> HTTP Client</a></li><li class="chapter-item expanded "><a href="rpc.html"><strong aria-hidden="true">10.</strong> RPC</a></li><li class="chapter-item expanded "><a href="storage.html"><strong aria-hidden="true">11.</strong> Storage</a></li><li class="chapter-item expanded "><a href="validator.html"><strong aria-hidden="true">12.</strong> Validator</a></li><li class="chapter-item expanded "><a href="transaction_state.html"><strong aria-hidden="true">13.</strong> Transaction State</a></li><li class="chapter-item expanded "><a href="indexer.html"><strong aria-hidden="true">14.</strong> Indexer</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString();
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
