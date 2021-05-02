<template>
  <div id="app">
    <Header />
    <router-view />
    <particles-bg  color="#cccccc" num=150 type="cobweb" :bg="true"/>
  </div>
</template>

<script>
import { ParticlesBg } from "particles-bg-vue";
import Header from "@/components/Header";

export default {
  name: "App",
  components: {
    Header,
    ParticlesBg,
  },
  watch: {
        $route: {
            immediate: true,
            handler(to, from) {
                document.title = this.$route.name || 'Sybl';
            }
        },
    }
};
</script>
