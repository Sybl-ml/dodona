<template>
  <div id="app">
    <Header />
    <router-view />
  </div>
</template>

<script>
import Header from "@/components/Header";
export default {
  name: "App",
  components: {
    Header
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
