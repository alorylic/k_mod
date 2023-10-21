<script setup>
import { ref, onMounted, h } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";
import { Notification } from '@arco-design/web-vue';

const mods = ref([]);
const gamePath = ref("");
const columns = [
  {
    title: '名称',
    dataIndex: 'name',
    key: 'name',
  },
  {
    title: '插入时间',
    dataIndex: 'insert_time',
    key: 'insert_time',
    width: 200
  },
  {
    title: '状态',
    slotName: 'apply',
    width: 60
  },
  {
    title: '编辑',
    slotName: 'del',
    width: 60
  }
]
const scroll = {
  x: '100%',
  y: '100%'
};
const rowSelection = ref({
      type: 'checkbox',
      showCheckedAll: true,
      onlyCurrent: false,
    });
const selectMods = ref([]);

async function refreshMods() {
  const data = await invoke("get_mods");
  mods.value = data;
};

onMounted(async () => {
  await refreshMods();
  await invoke("get_game_path").then( (path) => {
    gamePath.value = path;
  })
  .catch( (err) => {
    Notification.err({
      title: '获取游戏路径失败',
      content: JSON.stringify(err),
    })
  })
});


async function addMod() {
  const dir = await open({
    directory: true,
  });
  if (dir == null) {
    return;
  }
  await invoke("add_mod", {path: dir})
  .then( (mod) => {
    mods.value.push(mod);
  })
  .catch( (err) => {
    Notification.err({
      title: '添加mod失败',
      content: JSON.stringify(err),
    })
  })
};

async function delMod(modInfo) {
  console.log(modInfo)
  if (modInfo.apply) {
    Notification.error({
      title: '请先取消使用该mod',
      content: '请先取消使用该mod',
    })
    return;
  }
  await invoke("remove_mod", { modInfo: modInfo })
}

async function changeModStatus(modInfo, status) {
  if (status) {
    await invoke("use_the_mod", { modInfo: modInfo })
  } else {
    await invoke("unuse_mod", { modInfo: modInfo })
  }
  const info = mods.value.find( (item) => item.name == modInfo.name);
  info.apply = status;
  mods.value = [...mods.value];
  console.log(mods, status);
}

async function setGamePath() {
  const dir = await open({
    directory: true,
  });
  if (dir == null) {
    return;
  }
  await invoke("set_game_path", {path: dir})
  .then( (path) => {
    gamePath.value = path;
  })
  .catch( (err) => {
    Notification.err({
      title: '设置游戏路径失败',
      content: JSON.stringify(err),
    })
  })
}

async function clearBackupFile() {
  await invoke("clear_game_file_backup")
}
</script>

<template>
  <div>
    <div class="table_content">
      <a-table
      row-key="name"
      :scroll="scroll"
      :scrollbar="true"
      :columns="columns"
      :data="mods"
      :row-selection="rowSelection"
      v-model:selectedKeys="selectMods">

        <template #apply="{ record }">
          <a-switch size="small" :model-value="record.apply" @change="value => changeModStatus(record, value)" />
        </template>
        <template #del="{ record }">
          <icon-delete @click="delMod(record)" />
        </template>
    </a-table>
    </div>

    <footer class="bottom">
      <div class="set_game_path">
        <a-tag>{{ gamePath }}</a-tag>
        <a-button style="margin-left: 5px;" size="mini" type="primary" @click="setGamePath">设置MOD安装路径</a-button>
      </div>
      <div class="button_group">
        <a-button size="mini" type="primary" @click="addMod">导入mod</a-button>
        <a-button size="mini" type="primary" @click="clearBackupFile">清除游戏文件备份</a-button>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.bottom {
  width: calc(100% - 6px);
  display: flex;
  justify-content: space-between;
  align-items: center;
  position: absolute;
  bottom: 10px;
}

.game_path {
  display: flex;
  flex-direction: column;
}

.button_group > * {
  margin-left: 3px;
}

.table_content {
  height: 93vh;
}
</style>