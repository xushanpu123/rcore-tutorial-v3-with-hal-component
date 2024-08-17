(function() {var type_impls = {
"os":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-MmioTransport\" class=\"impl\"><a href=\"#impl-MmioTransport\" class=\"anchor\">§</a><h3 class=\"code-header\">impl MmioTransport</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.new\" class=\"method\"><h4 class=\"code-header\">pub unsafe fn <a class=\"fn\">new</a>(\n    header: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/ptr/non_null/struct.NonNull.html\" title=\"struct core::ptr::non_null::NonNull\">NonNull</a>&lt;VirtIOHeader&gt;\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;MmioTransport, MmioError&gt;</h4></section></summary><div class=\"docblock\"><p>Constructs a new VirtIO MMIO transport, or returns an error if the header reports an\nunsupported version.</p>\n<h5 id=\"safety\"><a href=\"#safety\">Safety</a></h5>\n<p><code>header</code> must point to a properly aligned valid VirtIO MMIO region, which must remain valid\nfor the lifetime of the transport that is returned.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.version\" class=\"method\"><h4 class=\"code-header\">pub fn <a class=\"fn\">version</a>(&amp;self) -&gt; MmioVersion</h4></section></summary><div class=\"docblock\"><p>Gets the version of the VirtIO MMIO transport.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.vendor_id\" class=\"method\"><h4 class=\"code-header\">pub fn <a class=\"fn\">vendor_id</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.u32.html\">u32</a></h4></section></summary><div class=\"docblock\"><p>Gets the vendor ID.</p>\n</div></details></div></details>",0,"os::drivers::block::virtio_blk::VirtIoTransport","os::drivers::gpu::VirtIoTransport","os::drivers::input::VirtIoTransport"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Transport-for-MmioTransport\" class=\"impl\"><a href=\"#impl-Transport-for-MmioTransport\" class=\"anchor\">§</a><h3 class=\"code-header\">impl Transport for MmioTransport</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.device_type\" class=\"method trait-impl\"><a href=\"#method.device_type\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">device_type</a>(&amp;self) -&gt; DeviceType</h4></section></summary><div class='docblock'>Gets the device type.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.read_device_features\" class=\"method trait-impl\"><a href=\"#method.read_device_features\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">read_device_features</a>(&amp;mut self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.u64.html\">u64</a></h4></section></summary><div class='docblock'>Reads device features.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.write_driver_features\" class=\"method trait-impl\"><a href=\"#method.write_driver_features\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">write_driver_features</a>(&amp;mut self, driver_features: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.u64.html\">u64</a>)</h4></section></summary><div class='docblock'>Writes device features.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.max_queue_size\" class=\"method trait-impl\"><a href=\"#method.max_queue_size\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">max_queue_size</a>(&amp;mut self, queue: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.u16.html\">u16</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.u32.html\">u32</a></h4></section></summary><div class='docblock'>Gets the max size of the given queue.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.notify\" class=\"method trait-impl\"><a href=\"#method.notify\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">notify</a>(&amp;mut self, queue: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.u16.html\">u16</a>)</h4></section></summary><div class='docblock'>Notifies the given queue on the device.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.get_status\" class=\"method trait-impl\"><a href=\"#method.get_status\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">get_status</a>(&amp;self) -&gt; DeviceStatus</h4></section></summary><div class='docblock'>Gets the device status.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.set_status\" class=\"method trait-impl\"><a href=\"#method.set_status\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">set_status</a>(&amp;mut self, status: DeviceStatus)</h4></section></summary><div class='docblock'>Sets the device status.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.set_guest_page_size\" class=\"method trait-impl\"><a href=\"#method.set_guest_page_size\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">set_guest_page_size</a>(&amp;mut self, guest_page_size: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.u32.html\">u32</a>)</h4></section></summary><div class='docblock'>Sets the guest page size.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.requires_legacy_layout\" class=\"method trait-impl\"><a href=\"#method.requires_legacy_layout\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">requires_legacy_layout</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Returns whether the transport requires queues to use the legacy layout. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.queue_set\" class=\"method trait-impl\"><a href=\"#method.queue_set\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">queue_set</a>(\n    &amp;mut self,\n    queue: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.u16.html\">u16</a>,\n    size: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.u32.html\">u32</a>,\n    descriptors: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.usize.html\">usize</a>,\n    driver_area: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.usize.html\">usize</a>,\n    device_area: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.usize.html\">usize</a>\n)</h4></section></summary><div class='docblock'>Sets up the given queue.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.queue_unset\" class=\"method trait-impl\"><a href=\"#method.queue_unset\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">queue_unset</a>(&amp;mut self, queue: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.u16.html\">u16</a>)</h4></section></summary><div class='docblock'>Disables and resets the given queue.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.queue_used\" class=\"method trait-impl\"><a href=\"#method.queue_used\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">queue_used</a>(&amp;mut self, queue: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.u16.html\">u16</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Returns whether the queue is in use, i.e. has a nonzero PFN or is marked as ready.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.ack_interrupt\" class=\"method trait-impl\"><a href=\"#method.ack_interrupt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">ack_interrupt</a>(&amp;mut self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Acknowledges an interrupt. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.config_space\" class=\"method trait-impl\"><a href=\"#method.config_space\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">config_space</a>&lt;T&gt;(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/ptr/non_null/struct.NonNull.html\" title=\"struct core::ptr::non_null::NonNull\">NonNull</a>&lt;T&gt;, Error&gt;</h4></section></summary><div class='docblock'>Gets the pointer to the config space.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.begin_init\" class=\"method trait-impl\"><a href=\"#method.begin_init\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">begin_init</a>&lt;F&gt;(&amp;mut self, supported_features: F) -&gt; F<span class=\"where fmt-newline\">where\n    F: Flags&lt;Bits = <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.u64.html\">u64</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/bit/trait.BitAnd.html\" title=\"trait core::ops::bit::BitAnd\">BitAnd</a>&lt;Output = F&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,</span></h4></section></summary><div class='docblock'>Begins initializing the device. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.finish_init\" class=\"method trait-impl\"><a href=\"#method.finish_init\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">finish_init</a>(&amp;mut self)</h4></section></summary><div class='docblock'>Finishes initializing the device.</div></details></div></details>","Transport","os::drivers::block::virtio_blk::VirtIoTransport","os::drivers::gpu::VirtIoTransport","os::drivers::input::VirtIoTransport"],["<section id=\"impl-Sync-for-MmioTransport\" class=\"impl\"><a href=\"#impl-Sync-for-MmioTransport\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for MmioTransport</h3></section>","Sync","os::drivers::block::virtio_blk::VirtIoTransport","os::drivers::gpu::VirtIoTransport","os::drivers::input::VirtIoTransport"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Drop-for-MmioTransport\" class=\"impl\"><a href=\"#impl-Drop-for-MmioTransport\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for MmioTransport</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.drop\" class=\"method trait-impl\"><a href=\"#method.drop\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html#tymethod.drop\" class=\"fn\">drop</a>(&amp;mut self)</h4></section></summary><div class='docblock'>Executes the destructor for this type. <a href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html#tymethod.drop\">Read more</a></div></details></div></details>","Drop","os::drivers::block::virtio_blk::VirtIoTransport","os::drivers::gpu::VirtIoTransport","os::drivers::input::VirtIoTransport"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-MmioTransport\" class=\"impl\"><a href=\"#impl-Debug-for-MmioTransport\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for MmioTransport</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Error.html\" title=\"struct core::fmt::Error\">Error</a>&gt;</h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","os::drivers::block::virtio_blk::VirtIoTransport","os::drivers::gpu::VirtIoTransport","os::drivers::input::VirtIoTransport"],["<section id=\"impl-Send-for-MmioTransport\" class=\"impl\"><a href=\"#impl-Send-for-MmioTransport\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for MmioTransport</h3></section>","Send","os::drivers::block::virtio_blk::VirtIoTransport","os::drivers::gpu::VirtIoTransport","os::drivers::input::VirtIoTransport"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()