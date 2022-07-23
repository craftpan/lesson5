#![cfg_attr(not(feature = "std"), no_std)]

/// A module for proof of existence
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn proofs)]
    pub type Proofs<T: Config> =
        StorageMap<_, Blake2_128Concat, Vec<u8>, (T::AccountId, T::BlockNumber)>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ClaimCreated(T::AccountId, Vec<u8>),
        ClaimRevoked(T::AccountId, Vec<u8>),
        ClaimTransferred(T::AccountId, T::AccountId, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        ProofAlreadyExist,
        ClaimNotExist,
        NotClaimOwner,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
            //存证申明人
            let sender = ensure_signed(origin)?;

            // 查看是否有重复申明的情况
            ensure!(
                !Proofs::<T>::contains_key(&claim),
                Error::<T>::ProofAlreadyExist
            );

            // 存储存证申明信息
            Proofs::<T>::insert(
                &claim,
                (sender.clone(), frame_system::Pallet::<T>::block_number()),
            );

            // 存证创建成功事件
            Self::deposit_event(Event::ClaimCreated(sender, claim));

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
            // 验证交易发起人
            let sender = ensure_signed(origin)?;

            // 获取存证申明人
            let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

            // 验证存证申明人是否是交易发起人
            ensure!(owner == sender, Error::<T>::NotClaimOwner);

            // 删除该存证
            Proofs::<T>::remove(&claim);

            // 存证删除成功事件
            Self::deposit_event(Event::ClaimRevoked(sender, claim));

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn transfer_claim(
            origin: OriginFor<T>,
            receiver: T::AccountId,
            claim: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            // 验证交易发起人
            let sender = ensure_signed(origin)?;

            // 获取存证申请人
            let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

            // 验证存证申请人是否是交易发起人
            ensure!(owner == sender, Error::<T>::NotClaimOwner);

            // 转移存证，更新存证申请人
            Proofs::<T>::insert(
                &claim,
                (receiver.clone(), frame_system::Pallet::<T>::block_number()),
            );

            // 存证转移成功事件
            Self::deposit_event(Event::ClaimTransferred(sender, receiver, claim));

            Ok(().into())
        }
    }
}
