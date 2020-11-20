<template>
  <div>{{ privkey }}</div>
</template>

<script>
export default {
  name: "ClientDashboard",
  props: {
    privkey: String,
  },
};
</script>
