<template>
  <div>{{ privKey }}</div>
</template>

<script>
export default {
  name: "ClientDashboard",
  props: {
    privKey: String,
  },
};
</script>
